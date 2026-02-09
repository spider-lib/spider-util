//! Response types for the spider-lib framework.
//!
//! This module provides abstractions for handling both regular and stream responses.

use crate::response::Response;
#[cfg(feature = "stream")]
use crate::stream_response::StreamResponse;
use std::fmt::Debug;

/// An enum that represents either a regular response or a stream response.
/// This allows the framework to handle both response types in a unified way.
#[derive(Debug)]
pub enum ResponseType {
    /// A regular response with the full body loaded in memory.
    Regular(Response),

    /// A stream response where the body is processed as a stream.
    #[cfg(feature = "stream")]
    Stream(StreamResponse),
}

impl ResponseType {
    /// Converts a regular response to the enum.
    pub fn from_regular(response: Response) -> Self {
        ResponseType::Regular(response)
    }

    /// Gets the URL from the response regardless of type.
    pub fn url(&self) -> &url::Url {
        match self {
            ResponseType::Regular(response) => &response.url,
            #[cfg(feature = "stream")]
            ResponseType::Stream(response) => &response.url,
        }
    }

    /// Gets the status code from the response regardless of type.
    pub fn status(&self) -> http::StatusCode {
        match self {
            ResponseType::Regular(response) => response.status,
            #[cfg(feature = "stream")]
            ResponseType::Stream(response) => response.status,
        }
    }

    /// Gets the headers from the response regardless of type.
    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        match self {
            ResponseType::Regular(response) => &response.headers,
            #[cfg(feature = "stream")]
            ResponseType::Stream(response) => &response.headers,
        }
    }

    /// Converts a stream response to the enum.
    #[cfg(feature = "stream")]
    pub fn from_stream(response: StreamResponse) -> Self {
        ResponseType::Stream(response)
    }

    /// Converts the response to a regular response.
    /// For stream responses, this will consume the stream and collect all data.
    #[cfg(feature = "stream")]
    pub async fn to_regular(self) -> Result<Response, crate::error::SpiderError> {
        match self {
            ResponseType::Regular(response) => Ok(response),
            ResponseType::Stream(stream_response) => stream_response
                .to_response()
                .await
                .map_err(|e| crate::error::SpiderError::IoError(e.to_string())),
        }
    }

    /// Converts the response to a stream response.
    /// For regular responses, this will wrap the body in a stream.
    #[cfg(feature = "stream")]
    pub async fn to_stream(self) -> Result<StreamResponse, crate::error::SpiderError> {
        match self {
            ResponseType::Regular(response) => response
                .to_stream_response()
                .await
                .map_err(|e| crate::error::SpiderError::IoError(e.to_string())),
            ResponseType::Stream(stream_response) => Ok(stream_response),
        }
    }

    /// Gets the request URL from the response regardless of type.
    pub fn request_url(&self) -> &url::Url {
        match self {
            ResponseType::Regular(response) => &response.request_url,
            #[cfg(feature = "stream")]
            ResponseType::Stream(response) => &response.request_url,
        }
    }
}

