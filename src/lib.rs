//! # spider-util
//!
//! Utility types and traits for the `spider-lib` framework.
//!
//! ## Example
//!
//! ```rust,ignore
//! use spider_util::{request::Request, item::ScrapedItem};
//! use url::Url;
//!
//! let url = Url::parse("https://example.com").unwrap();
//! let request = Request::new(url);
//!
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
