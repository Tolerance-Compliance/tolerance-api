//! Offline ingest tool (native binary).
//!
//! Parses the NIST 800-171 JSON files (r1/r2/r3), builds the search index and
//! the assembled response shapes once, and emits Workers KV bulk-put payload
//! files. Run it locally / in CI; it never runs on the Worker.
//!
//! Usage:
//!   cargo run --bin ingest                 # writes ./kv-bulk/bulk-*.json
//!   wrangler kv bulk put ./kv-bulk/bulk-0000.json --binding DOCS --remote
//!   (repeat for each generated file)

use std::collections::HashMap;
use std::fs;

use serde::Serialize;

use tolerance_api::cmmc::assemble::{build_family, build_requirement, build_security_requirement};
use tolerance_api::cmmc::index::SearchIndex;
use tolerance_api::cmmc::model::{
    Document, DocumentKey, DocumentRevision, Element, ElementType, NistData, NistDocument,
    Relationship,
};
use tolerance_api::cmmc::poam::PoamValidator;
use tolerance_api::cmmc::response::{
    DataSummary, DocumentInfo, Family, Requirement, SecurityRequirement,
};
use tolerance_api::cmmc::scoring::ScoringDatabase;
use tolerance_api::kv::keys::{self, PageManifest, ELEMENTS_PER_PAGE};

/// One KV key/value pair in the `wrangler kv bulk put` file format.
#[derive(Serialize)]
struct BulkPair {
    key: String,
    value: String,
}

/// Max pairs per bulk file (wrangler accepts up to 10,000; stay conservative).
const PAIRS_PER_FILE: usize = 5_000;
const OUTPUT_DIR: &str = "kv-bulk";

struct Spec {
    key: DocumentKey,
    path: &'static str,
}

fn specs() -> Vec<Spec> {
    vec![
        Spec {
            key: DocumentKey::nist(NistDocument::Sp800171, DocumentRevision::Rev1),
            path: "data/cprt-sp_800_171_1_0_0.json",
        },
        Spec {
            key: DocumentKey::nist(NistDocument::Sp800171, DocumentRevision::Rev2),
            path: "data/cprt-sp_800_171_2_0_0.json",
        },
        Spec {
            key: DocumentKey::nist(NistDocument::Sp800171, DocumentRevision::Rev3),
            path: "data/cprt-sp_800_171_3_0_0-20260215-171034.json",
        },
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scoring = ScoringDatabase::new();
    let poam = PoamValidator::new(ScoringDatabase::new());

    let mut pairs: Vec<BulkPair> = Vec::new();
    let mut doc_infos: Vec<DocumentInfo> = Vec::new();

    for spec in specs() {
        let contents = fs::read_to_string(spec.path)
            .map_err(|e| format!("failed to read {}: {}", spec.path, e))?;
        let data: NistData = serde_json::from_str(&contents)
            .map_err(|e| format!("failed to parse {}: {}", spec.path, e))?;

        eprintln!("Ingesting {} ({})", spec.key, spec.path);
        emit_document(&mut pairs, spec.key, &data, &scoring, &poam)?;

        doc_infos.push(DocumentInfo {
            id: spec.key.to_string(),
            name: spec.key.display_name(),
            document: spec.key.document_string(),
            revision: spec.key.revision_string(),
        });
    }

    // Global documents index.
    push(&mut pairs, keys::index(), &doc_infos)?;

    write_bulk_files(&pairs)?;
    Ok(())
}

fn emit_document(
    pairs: &mut Vec<BulkPair>,
    key: DocumentKey,
    data: &NistData,
    scoring: &ScoringDatabase,
    poam: &PoamValidator,
) -> Result<(), Box<dyn std::error::Error>> {
    let elements: &[Element] = &data.response.elements.elements;
    let relationships: &[Relationship] = &data.response.elements.relationships;
    let index = SearchIndex::build(elements);

    // --- summary -----------------------------------------------------------
    let summary = DataSummary {
        document: data
            .response
            .elements
            .documents
            .first()
            .cloned()
            .unwrap_or_else(|| Document {
                doc_identifier: String::new(),
                name: String::new(),
                version: String::new(),
                website: String::new(),
            }),
        family_count: index.count_by_type(ElementType::Family),
        requirement_count: index.count_by_type(ElementType::Requirement),
        security_requirement_count: index.count_by_type(ElementType::SecurityRequirement),
        relationship_count: relationships.len(),
    };
    push(pairs, keys::summary(key), &summary)?;

    // --- families (full list + per-family) ---------------------------------
    let families: Vec<Family> = index
        .get_by_type(ElementType::Family)
        .iter()
        .filter_map(|&i| elements.get(i))
        .map(|f| build_family(f, elements, scoring, poam))
        .collect();
    push(pairs, keys::families(key), &families)?;
    for family in &families {
        push(pairs, keys::family(key, &family.identifier), family)?;
    }

    // --- requirements / security-requirements (full lists) -----------------
    let requirements: Vec<Requirement> = index
        .get_by_type(ElementType::Requirement)
        .iter()
        .filter_map(|&i| elements.get(i))
        .map(|r| build_requirement(r, elements, scoring, poam))
        .collect();
    push(pairs, keys::requirements(key), &requirements)?;

    let secreq: Vec<SecurityRequirement> = index
        .get_by_type(ElementType::SecurityRequirement)
        .iter()
        .filter_map(|&i| elements.get(i))
        .map(|s| build_security_requirement(s, elements, scoring, poam))
        .collect();
    push(pairs, keys::secreq(key), &secreq)?;

    // --- relationships (full list) -----------------------------------------
    push(pairs, keys::relationships(key), &relationships.to_vec())?;

    // --- per-element keys + per-element relationships ----------------------
    // `by_identifier` deduplicates ids exactly like the old runtime did
    // (last element with a given id wins).
    for (id, &idx) in &index.by_identifier {
        if let Some(el) = elements.get(idx) {
            push(pairs, keys::element(key, id), el)?;
        }
        let rels: Vec<&Relationship> = relationships
            .iter()
            .filter(|r| &r.source_element_identifier == id || &r.dest_element_identifier == id)
            .collect();
        push(pairs, keys::element_rels(key, id), &rels)?;
    }

    // --- paged elements: "all" bucket --------------------------------------
    let all: Vec<&Element> = elements.iter().collect();
    write_pages(pairs, key, "all", &all)?;

    // --- paged elements + id lists: per type bucket ------------------------
    for (et, idxs) in &index.by_type {
        let bucket = et.slug();
        let bucket_elems: Vec<&Element> =
            idxs.iter().filter_map(|&i| elements.get(i)).collect();
        write_pages(pairs, key, bucket, &bucket_elems)?;

        let ids: Vec<&str> = bucket_elems
            .iter()
            .map(|e| e.element_identifier.as_str())
            .collect();
        push(pairs, keys::ids(key, bucket), &ids)?;
    }

    // --- inverted index posting lists --------------------------------------
    for (token, idx_set) in &index.inverted_index {
        // Preserve document order and dedupe identifiers.
        let mut sorted: Vec<usize> = idx_set.iter().copied().collect();
        sorted.sort_unstable();
        let mut seen: HashMap<&str, ()> = HashMap::new();
        let ids: Vec<&str> = sorted
            .iter()
            .filter_map(|&i| elements.get(i))
            .map(|e| e.element_identifier.as_str())
            .filter(|id| seen.insert(id, ()).is_none())
            .collect();
        push(pairs, keys::token(key, token), &ids)?;
    }

    Ok(())
}

fn write_pages(
    pairs: &mut Vec<BulkPair>,
    key: DocumentKey,
    bucket: &str,
    items: &[&Element],
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = PageManifest {
        total: items.len(),
        page_size: ELEMENTS_PER_PAGE,
        page_count: items.len().div_ceil(ELEMENTS_PER_PAGE),
    };
    push(pairs, keys::elements_manifest(key, bucket), &manifest)?;

    for (n, chunk) in items.chunks(ELEMENTS_PER_PAGE).enumerate() {
        push(pairs, keys::elements_page(key, bucket, n), &chunk.to_vec())?;
    }
    Ok(())
}

fn push<T: Serialize>(
    pairs: &mut Vec<BulkPair>,
    key: String,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    pairs.push(BulkPair {
        key,
        value: serde_json::to_string(value)?,
    });
    Ok(())
}

fn write_bulk_files(pairs: &[BulkPair]) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTPUT_DIR)?;

    let mut file_count = 0;
    for (n, chunk) in pairs.chunks(PAIRS_PER_FILE).enumerate() {
        let path = format!("{}/bulk-{:04}.json", OUTPUT_DIR, n);
        fs::write(&path, serde_json::to_string(chunk)?)?;
        eprintln!("Wrote {} ({} pairs)", path, chunk.len());
        file_count += 1;
    }

    eprintln!(
        "\nDone: {} pairs across {} file(s) in ./{}/",
        pairs.len(),
        file_count,
        OUTPUT_DIR
    );
    eprintln!("Upload with, for each file:");
    eprintln!("  wrangler kv bulk put ./{}/bulk-0000.json --binding DOCS --remote", OUTPUT_DIR);
    Ok(())
}
