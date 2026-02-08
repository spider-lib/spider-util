//! # spider-util
//!
//! Provides utility types, traits, and implementations for the `spider-lib` framework.
//!
//! ## Overview
//!
//! The `spider-util` crate contains fundamental data structures, error types,
//! and utility functions that are shared across all components of the spider
//! framework. This crate serves as the common foundation for all other spider
//! crates, providing the basic building blocks for web scraping operations.
//!
//! ## Key Components
//!
//! - **Request**: Represents an HTTP request with URL, method, headers, and body
//! - **Response**: Represents an HTTP response with status, headers, and body
//! - **ScrapedItem**: Trait and derive macro for defining data structures to hold scraped data
//! - **Error Handling**: Comprehensive error types for all operations
//! - **Bloom Filter**: Efficient probabilistic data structure for duplicate detection
//! - **Utilities**: Helper functions and extensions for common operations
//!
//! ## Architecture
//!
//! This crate is designed to be lightweight and reusable, containing only the
//! essential types and utilities needed by other spider components. It has minimal
//! external dependencies to ensure stability and compatibility.
//!
//! ## Example
//!
//! ```rust,ignore
//! use spider_util::{request::Request, response::Response, item::ScrapedItem};
//! use url::Url;
//!
//! // Create a request
//! let url = Url::parse("https://example.com").unwrap();
//! let request = Request::new(url);
//!
//! // Define a scraped item
//! #[spider_macro::scraped_item]
//! struct Article {
//!     title: String,
//!     content: String,
//! }
//! ```

pub mod bloom_filter;
pub mod error;
pub mod item;
pub mod request;
pub mod response;
pub mod utils;

// Re-export serde and serde_json for use in macros
pub use serde;
pub use serde_json;
