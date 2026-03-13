#![allow(
    clippy::must_use_candidate,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod dtag;
pub mod error;
pub mod header;
pub mod item;
pub mod query;

pub use error::DcoslError;
