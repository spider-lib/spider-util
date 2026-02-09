//! Response types for the spider-lib framework.
//!
//! This module provides abstractions for handling both regular and streaming responses.

use crate::response::Response;
#[cfg(feature = "streaming")]
use crate::streaming_response::StreamingResponse;
use std::fmt::Debug;

/// An enum that represents either a regular response or a streaming response.
/// This allows the framework to handle both response types in a unified way.
#[derive(Debug)]
pub enum ResponseType {
    /// A regular response with the full body loaded in memory.
    Regular(Response),

    /// A streaming response where the body is processed as a stream.
    #[cfg(feature = "streaming")]
    Streaming(StreamingResponse),
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
            #[cfg(feature = "streaming")]
            ResponseType::Streaming(response) => &response.url,
        }
    }

    /// Gets the status code from the response regardless of type.
    pub fn status(&self) -> http::StatusCode {
        match self {
            ResponseType::Regular(response) => response.status,
            #[cfg(feature = "streaming")]
            ResponseType::Streaming(response) => response.status,
        }
    }

    /// Gets the headers from the response regardless of type.
    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        match self {
            ResponseType::Regular(response) => &response.headers,
            #[cfg(feature = "streaming")]
            ResponseType::Streaming(response) => &response.headers,
        }
    }

    /// Converts a streaming response to the enum.
    #[cfg(feature = "streaming")]
    pub fn from_streaming(response: StreamingResponse) -> Self {
        ResponseType::Streaming(response)
    }

    /// Converts the response to a regular response.
    /// For streaming responses, this will consume the stream and collect all data.
    #[cfg(feature = "streaming")]
    pub async fn to_regular(self) -> Result<Response, crate::error::SpiderError> {
        match self {
            ResponseType::Regular(response) => Ok(response),
            ResponseType::Streaming(streaming_response) => streaming_response
                .to_response()
                .await
                .map_err(|e| crate::error::SpiderError::IoError(e.to_string())),
        }
    }

    /// Converts the response to a streaming response.
    /// For regular responses, this will wrap the body in a stream.
    #[cfg(feature = "streaming")]
    pub async fn to_streaming(self) -> Result<StreamingResponse, crate::error::SpiderError> {
        match self {
            ResponseType::Regular(response) => response
                .to_streaming_response()
                .await
                .map_err(|e| crate::error::SpiderError::IoError(e.to_string())),
            ResponseType::Streaming(streaming_response) => Ok(streaming_response),
        }
    }

    /// Gets the request URL from the response regardless of type.
    pub fn request_url(&self) -> &url::Url {
        match self {
            ResponseType::Regular(response) => &response.request_url,
            #[cfg(feature = "streaming")]
            ResponseType::Streaming(response) => &response.request_url,
        }
    }
}

