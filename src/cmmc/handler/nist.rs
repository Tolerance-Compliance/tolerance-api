//! /v1/nist/:document/:revision/*
//!
//! Thin KV readers: every response is a pre-rendered fragment produced by the
//! offline ingest tool. Handlers map the request to a key and return the value.

use std::collections::HashSet;

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
};

use crate::cmmc::format_response::{wants_toon, FormatResponse};
use crate::cmmc::index::tokenize;
use crate::cmmc::model::{Element, Relationship};
use crate::cmmc::response::{
    DataSummary, DocumentInfo, Family, PaginatedResponse, Requirement, SecurityRequirement,
};
use crate::handler::error::ApiError;
use crate::kv::keys;

use super::query::{parse_nist_document_key, require_cmmc_structured, ElementQuery};
use super::util::{paginated_elements, require};
use crate::kv::store::AppState;

/// Get list of available documents.
#[utoipa::path(
    get,
    path = "/v1/nist/documents",
    responses(
        (status = 200, description = "List of available documents. Send `Accept: text/toon` for LLM-optimized output (30-40% fewer tokens).",
         content(
             (Vec<DocumentInfo> = "application/json"),
             (String = "text/toon")
         ))
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<DocumentInfo>>, ApiError> {
    let docs: Vec<DocumentInfo> = state.get_json(&keys::index()).await?.unwrap_or_default();
    Ok(FormatResponse::with_format(docs, wants_toon(&headers)))
}

/// Get summary for a specific document+revision.
#[utoipa::path(
    get,
    operation_id = "nist_get_summary",
    path = "/v1/nist/{document}/{revision}/summary",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`")
    ),
    responses(
        (status = 200, description = "Summary retrieved successfully. Send `Accept: text/toon` for LLM-optimized output (30-40% fewer tokens).",
         content(
             (DataSummary = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_summary(
    State(state): State<AppState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<DataSummary>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let summary: DataSummary = require(
        &state,
        &keys::summary(key),
        format!("Document {} not loaded", key),
    )
    .await?;
    Ok(FormatResponse::with_format(summary, wants_toon(&headers)))
}

/// Get all families.
#[utoipa::path(
    get,
    operation_id = "nist_get_families",
    path = "/v1/nist/{document}/{revision}/families",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`")
    ),
    responses(
        (status = 200, description = "List of families.",
         content(
             (Vec<Family> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_families(
    State(state): State<AppState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Family>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let families: Vec<Family> = require(
        &state,
        &keys::families(key),
        format!("Document {} not loaded", key),
    )
    .await?;
    Ok(FormatResponse::with_format(families, wants_toon(&headers)))
}

/// Get a specific family by identifier.
#[utoipa::path(
    get,
    operation_id = "nist_get_family",
    path = "/v1/nist/{document}/{revision}/families/{id}",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`"),
        ("id" = String, Path, description = "Family identifier")
    ),
    responses(
        (status = 200, description = "Family details.",
         content(
             (Family = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Family not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_family(
    State(state): State<AppState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Family>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let family: Family = require(
        &state,
        &keys::family(key, &id),
        format!("Family '{}' not found", id),
    )
    .await?;
    Ok(FormatResponse::with_format(family, wants_toon(&headers)))
}

/// Get all elements (paginated; supports `?type=` and `?search=`).
#[utoipa::path(
    get,
    operation_id = "nist_get_elements",
    path = "/v1/nist/{document}/{revision}/elements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`"),
        ElementQuery
    ),
    responses(
        (status = 200, description = "Paginated list of elements.",
         content(
             (PaginatedResponse<Element> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_elements(
    State(state): State<AppState>,
    Path((document, revision)): Path<(String, String)>,
    Query(query): Query<ElementQuery>,
    headers: HeaderMap,
) -> Result<FormatResponse<PaginatedResponse<Element>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let element_type = query.parse_element_type();

    if let Some(term) = query.search.clone() {
        let resp = search_elements(&state, key, &term, element_type, query.offset(), query.limit())
            .await?;
        return Ok(FormatResponse::with_format(resp, wants_toon(&headers)));
    }

    let bucket = element_type.map(|t| t.slug()).unwrap_or("all");
    let resp = paginated_elements(&state, key, bucket, query.offset(), query.limit()).await?;
    Ok(FormatResponse::with_format(resp, wants_toon(&headers)))
}

/// Token-based search over precomputed posting lists.
///
/// Multi-word queries use AND semantics across token posting lists. Substring /
/// partial matching from the in-memory implementation is intentionally dropped:
/// it would require scanning the full document, defeating the KV sharding.
async fn search_elements(
    state: &AppState,
    key: crate::cmmc::model::DocumentKey,
    term: &str,
    element_type: Option<crate::cmmc::model::ElementType>,
    offset: usize,
    limit: usize,
) -> Result<PaginatedResponse<Element>, ApiError> {
    let tokens = tokenize(&term.to_lowercase());

    // Ordered candidate ids (doc order preserved by ingest), narrowed by AND.
    let mut candidates: Vec<String> = Vec::new();
    let mut other_sets: Vec<HashSet<String>> = Vec::new();

    for (i, tok) in tokens.iter().enumerate() {
        let posting: Vec<String> = state.get_json(&keys::token(key, tok)).await?.unwrap_or_default();
        if i == 0 {
            candidates = posting;
        } else {
            other_sets.push(posting.into_iter().collect());
        }
    }

    if !tokens.is_empty() {
        candidates.retain(|id| other_sets.iter().all(|s| s.contains(id)));
    }

    // Optional type filter via the precomputed per-type id list.
    if let Some(et) = element_type {
        let type_ids: HashSet<String> = state
            .get_json::<Vec<String>>(&keys::ids(key, et.slug()))
            .await?
            .unwrap_or_default()
            .into_iter()
            .collect();
        candidates.retain(|id| type_ids.contains(id));
    }

    let total = candidates.len();
    let capped_limit = limit.min(keys::SEARCH_PAGE_SIZE);

    let page_ids: Vec<String> = candidates
        .into_iter()
        .skip(offset)
        .take(capped_limit)
        .collect();

    let mut data: Vec<Element> = Vec::with_capacity(page_ids.len());
    for id in &page_ids {
        if let Some(el) = state.get_json::<Element>(&keys::element(key, id)).await? {
            data.push(el);
        }
    }

    Ok(PaginatedResponse {
        data,
        total,
        limit: capped_limit,
        offset,
        has_more: offset + capped_limit < total,
    })
}

/// Get a specific element by identifier.
#[utoipa::path(
    get,
    operation_id = "nist_get_element",
    path = "/v1/nist/{document}/{revision}/elements/{id}",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element details.",
         content(
             (Element = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Element not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_element(
    State(state): State<AppState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Element>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let element: Element = require(
        &state,
        &keys::element(key, &id),
        format!("Element '{}' not found", id),
    )
    .await?;
    Ok(FormatResponse::with_format(element, wants_toon(&headers)))
}

/// Get all requirements across all families.
#[utoipa::path(
    get,
    operation_id = "nist_get_requirements",
    path = "/v1/nist/{document}/{revision}/requirements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`")
    ),
    responses(
        (status = 200, description = "List of requirements.",
         content(
             (Vec<Requirement> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_requirements(
    State(state): State<AppState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Requirement>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let requirements: Vec<Requirement> = require(
        &state,
        &keys::requirements(key),
        format!("Document {} not loaded", key),
    )
    .await?;
    Ok(FormatResponse::with_format(requirements, wants_toon(&headers)))
}

/// Get all security requirements with discussion and assessment text.
#[utoipa::path(
    get,
    operation_id = "nist_get_security_requirements",
    path = "/v1/nist/{document}/{revision}/security-requirements",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`")
    ),
    responses(
        (status = 200, description = "List of security requirements.",
         content(
             (Vec<SecurityRequirement> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_security_requirements(
    State(state): State<AppState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<SecurityRequirement>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    require_cmmc_structured(key)?;
    let security_requirements: Vec<SecurityRequirement> = require(
        &state,
        &keys::secreq(key),
        format!("Document {} not loaded", key),
    )
    .await?;
    Ok(FormatResponse::with_format(
        security_requirements,
        wants_toon(&headers),
    ))
}

/// Get all relationships.
#[utoipa::path(
    get,
    operation_id = "nist_get_relationships",
    path = "/v1/nist/{document}/{revision}/relationships",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`")
    ),
    responses(
        (status = 200, description = "List of relationships.",
         content(
             (Vec<Relationship> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Document not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_relationships(
    State(state): State<AppState>,
    Path((document, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let relationships: Vec<Relationship> = require(
        &state,
        &keys::relationships(key),
        format!("Document {} not loaded", key),
    )
    .await?;
    Ok(FormatResponse::with_format(relationships, wants_toon(&headers)))
}

/// Get relationships for a specific element.
#[utoipa::path(
    get,
    operation_id = "nist_get_element_relationships",
    path = "/v1/nist/{document}/{revision}/elements/{id}/relationships",
    params(
        ("document" = String, Path, description = "NIST document: `sp800-171`"),
        ("revision" = String, Path, description = "Document revision: `r1`, `r2`, `r3`"),
        ("id" = String, Path, description = "Element identifier")
    ),
    responses(
        (status = 200, description = "Element relationships.",
         content(
             (Vec<Relationship> = "application/json"),
             (String = "text/toon")
         )),
        (status = 404, description = "Element not found")
    ),
    tag = "NIST"
)]
#[worker::send]
pub async fn get_element_relationships(
    State(state): State<AppState>,
    Path((document, revision, id)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<FormatResponse<Vec<Relationship>>, ApiError> {
    let key = parse_nist_document_key(&document, &revision)?;
    let relationships: Vec<Relationship> = require(
        &state,
        &keys::element_rels(key, &id),
        format!("Element '{}' not found", id),
    )
    .await?;
    Ok(FormatResponse::with_format(relationships, wants_toon(&headers)))
}
