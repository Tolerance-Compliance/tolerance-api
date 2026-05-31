//! Tolerance API core — framework-agnostic CMMC / NIST 800-171 logic.
//!
//! This crate holds the data models, search index, scoring/POA&M logic, request
//! parsing, response shaping, and content-negotiation encoding that are shared
//! between the native axum service and the Cloudflare Worker. It contains no web
//! framework, no async runtime, and no filesystem/network access, so it builds
//! identically for the native target and `wasm32-unknown-unknown`.
//!
//! The consuming crates are responsible only for transport concerns: routing,
//! extracting parameters, resolving a [`document::DocumentData`] (from disk on
//! native, from R2 in the Worker), and turning [`error::CoreError`] into an HTTP
//! response. Everything that determines the response *body* lives here, which is
//! what keeps the two deployments byte-for-byte identical.

pub mod document;
pub mod encode;
pub mod endpoints;
pub mod error;
pub mod index;
pub mod model;
pub mod poam;
pub mod query;
pub mod response;
pub mod scoring;
pub mod service;

// Commonly used re-exports.
pub use document::{DocumentData, DocumentContext};
pub use error::CoreError;
pub use index::SearchIndex;
pub use model::{
    DocumentKey, DocumentRevision, DocumentSource, Element, ElementType, FarDocument, NistData,
    NistDocument, NistDocumentKey, NistRevision, Relationship,
};
pub use poam::{PoamEligibility, PoamValidation, PoamValidator};
pub use response::{DataSummary, Family, PaginatedResponse, Requirement, SecurityRequirement};
pub use scoring::{CmmcLevel, Priority, RequirementScore, ScoringDatabase};
