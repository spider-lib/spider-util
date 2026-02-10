//! Data structures and utilities for handling HTTP responses in `spider-lib`.
//!
//! This module defines the `Response` struct, which represents an HTTP response
//! received from a web server. It encapsulates crucial information such as
//! the URL, status code, headers, and body of the response, along with any
//! associated metadata.
//!
//! Additionally, this module provides:
//! - Helper methods for `Response` to facilitate common tasks like parsing
//!   the body as HTML or JSON, and reconstructing the original `Request`.
//! - `Link` and `LinkType` enums for structured representation and extraction
//!   of hyperlinks found within the response content.

use crate::request::Request;
use crate::selector_cache::get_cached_selector;
use crate::utils;
use bytes::Bytes;
use dashmap::{DashMap, DashSet};
use linkify::{LinkFinder, LinkKind};
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use scraper::Html;
use serde::de::DeserializeOwned;
use serde_json::{self, Value};
use std::{borrow::Cow, str::Utf8Error, str::from_utf8};
use url::Url;

/// Represents the type of a discovered link.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkType {
    /// A link to another web page.
    Page,
    /// A link to a script file.
    Script,
    /// A link to a stylesheet.
    Stylesheet,
    /// A link to an image.
    Image,
    /// A link to a media file (audio/video).
    Media,
    /// A link to another type of resource.
    Other(String),
}

/// Represents a link discovered on a web page.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Link {
    /// The URL of the discovered link.
    pub url: Url,
    /// The type of the discovered link.
    pub link_type: LinkType,
}

/// Represents an HTTP response received from a server.
#[derive(Debug)]
pub struct Response {
    /// The final URL of the response after any redirects.
    pub url: Url,
    /// The HTTP status code of the response.
    pub status: StatusCode,
    /// The headers of the response.
    pub headers: HeaderMap,
    /// The body of the response.
    pub body: Bytes,
    /// The original URL of the request that led to this response.
    pub request_url: Url,
    /// Metadata associated with the response, carried over from the request.
    pub meta: DashMap<Cow<'static, str>, Value>,
    /// Indicates if the response was served from a cache.
    pub cached: bool,
}

impl Clone for Response {
    fn clone(&self) -> Self {
        Response {
            url: self.url.clone(),
            status: self.status,
            headers: self.headers.clone(),
            body: self.body.clone(),
            request_url: self.request_url.clone(),
            meta: self.meta.clone(),
            cached: self.cached,
        }
    }
}

impl Response {
    /// Reconstructs the original `Request` that led to this response.
    pub fn request_from_response(&self) -> Request {
        let mut request = Request::new(self.request_url.clone());
        request.meta = self.meta.clone();
        request
    }

    /// Deserializes the response body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Parses the response body as HTML.
    pub fn to_html(&self) -> Result<Html, Utf8Error> {
        let body_str = from_utf8(&self.body)?;
        Ok(Html::parse_document(body_str))
    }

    /// Lazily parses the response body as HTML, returning a closure that can be called when needed.
    pub fn lazy_html(&self) -> Result<impl Fn() -> Result<Html, Utf8Error> + '_, Utf8Error> {
        let body_bytes = &self.body;
        Ok(move || {
            let body_str = from_utf8(body_bytes)?;
            Ok(Html::parse_document(body_str))
        })
    }

    /// Extracts all unique, same-site links from the response body.
    pub fn links(&self) -> DashSet<Link> {
        let links = DashSet::new();

        if let Ok(html_fn) = self.lazy_html()
            && let Ok(html) = html_fn()
        {
            let selectors = vec![
                ("a[href]", "href"),
                ("link[href]", "href"),
                ("script[src]", "src"),
                ("img[src]", "src"),
                ("audio[src]", "src"),
                ("video[src]", "src"),
                ("source[src]", "src"),
            ];

            for (selector_str, attr_name) in selectors {
                if let Some(selector) = get_cached_selector(selector_str) {
                    for element in html.select(&selector) {
                        if let Some(attr_value) = element.value().attr(attr_name)
                            && let Ok(url) = self.url.join(attr_value)
                            && utils::is_same_site(&url, &self.url)
                        {
                            let link_type = match element.value().name() {
                                "a" => LinkType::Page,
                                "link" => {
                                    if let Some(rel) = element.value().attr("rel") {
                                        if rel.eq_ignore_ascii_case("stylesheet") {
                                            LinkType::Stylesheet
                                        } else {
                                            LinkType::Other(rel.to_string())
                                        }
                                    } else {
                                        LinkType::Other("link".to_string())
                                    }
                                }
                                "script" => LinkType::Script,
                                "img" => LinkType::Image,
                                "audio" | "video" | "source" => LinkType::Media,
                                _ => LinkType::Other(element.value().name().to_string()),
                            };
                            links.insert(Link { url, link_type });
                        }
                    }
                }
            }

            let finder = LinkFinder::new();
            for text_node in html.tree.values().filter_map(|node| node.as_text()) {
                for link in finder.links(text_node) {
                    if link.kind() == &LinkKind::Url
                        && let Ok(url) = self.url.join(link.as_str())
                        && utils::is_same_site(&url, &self.url)
                    {
                        links.insert(Link {
                            url,
                            link_type: LinkType::Page,
                        });
                    }
                }
            }
        }

        links
    }

}
