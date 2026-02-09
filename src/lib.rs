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
pub mod response_type;
pub mod selector_cache;
pub mod streaming_response;
pub mod utils;
