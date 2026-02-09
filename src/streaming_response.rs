//! Streaming response implementation for memory-efficient web scraping.
//!
//! This module provides streaming response capabilities that allow processing
//! of large responses without loading the entire body into memory at once.

use crate::response::{Link, LinkType, Response};
use bytes::Bytes;
use dashmap::DashMap;
use futures_util::StreamExt;
use futures_util::stream::Stream;
use http::StatusCode;
use reqwest::header::HeaderMap;
use scraper::Html;
use serde_json::Value;
use std::{borrow::Cow, pin::Pin};
use url::Url;

use std::fmt;

/// A streaming response that allows processing of large responses without
/// loading the entire body into memory at once.
pub struct StreamingResponse {
    /// The final URL of the response after any redirects.
    pub url: Url,
    /// The HTTP status code of the response.
    pub status: StatusCode,
    /// The headers of the response.
    pub headers: HeaderMap,
    /// The body of the response as a stream of Bytes chunks.
    pub body_stream: Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>,
    /// The original URL of the request that led to this response.
    pub request_url: Url,
    /// Metadata associated with the response, carried over from the request.
    pub meta: DashMap<Cow<'static, str>, Value>,
    /// Indicates if the response was served from a cache.
    pub cached: bool,
}

impl fmt::Debug for StreamingResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamingResponse")
            .field("url", &self.url)
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("request_url", &self.request_url)
            .field("cached", &self.cached)
            .finish()
    }
}

impl StreamingResponse {
    /// Converts the streaming response to a regular response by collecting all body chunks.
    /// This defeats the purpose of streaming but provides compatibility with existing code.
    pub async fn to_response(self) -> Result<Response, std::io::Error> {
        let mut body_bytes = Vec::new();
        let mut stream = self.body_stream;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            body_bytes.extend_from_slice(&chunk);
        }

        Ok(Response {
            url: self.url,
            status: self.status,
            headers: self.headers,
            body: bytes::Bytes::from(body_bytes),
            request_url: self.request_url,
            meta: self.meta,
            cached: self.cached,
        })
    }

    /// Provides a way to parse the streaming response as HTML by collecting chunks
    /// until enough data is available for parsing.
    /// Note: This consumes the streaming response to collect all data.
    pub async fn into_html(self) -> Result<Html, std::io::Error> {
        let mut body_bytes = Vec::new();
        let mut stream = self.body_stream;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            body_bytes.extend_from_slice(&chunk);
        }

        let body_str = std::str::from_utf8(&body_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Html::parse_document(body_str))
    }

    /// Extracts links from the streaming response by consuming and parsing the content.
    /// Note: This consumes the streaming response to collect all data.
    pub async fn into_links(self) -> Result<Vec<Link>, std::io::Error> {
        let base_url = self.url.clone();
        let html = self.into_html().await?;
        let mut links = Vec::new();

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
            if let Ok(selector) = scraper::Selector::parse(selector_str) {
                for element in html.select(&selector) {
                    if let Some(attr_value) = element.value().attr(attr_name)
                        && let Ok(url) = base_url.join(attr_value)
                    {
                        let link_type = match element.value().name() {
                            "a" => LinkType::Page,
                            "link" => LinkType::Stylesheet,
                            "script" => LinkType::Script,
                            "img" => LinkType::Image,
                            "audio" | "video" | "source" => LinkType::Media,
                            _ => LinkType::Other(element.value().name().to_string()),
                        };
                        links.push(Link { url, link_type });
                    }
                }
            }
        }

        Ok(links)
    }
}

