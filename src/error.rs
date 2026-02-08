//! Custom error types for the `spider-core` framework.
//!
//! This module defines a comprehensive set of custom error types, `SpiderError`
//! and `PipelineError`, used throughout the `spider-core` crate. These errors
//! encapsulate various issues that can occur during crawling, such as network
//! failures, URL parsing problems, I/O errors, configuration issues, and
//! problems within item processing pipelines.
//!
//! By centralizing error definitions, the module provides a consistent and
//! semantic way to report and handle errors, improving the robustness and
//! maintainability of the web scraping application.

use http;
use serde_json::Error as SerdeJsonError;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("Reqwest error: {message}")]
pub struct ReqwestErrorDetails {
    pub message: String,
    pub is_connect: bool,
    pub is_timeout: bool,
    // Add other relevant flags if necessary
}

impl From<reqwest::Error> for ReqwestErrorDetails {
    fn from(err: reqwest::Error) -> Self {
        ReqwestErrorDetails {
            is_connect: err.is_connect(),
            is_timeout: err.is_timeout(),
            message: err.to_string(),
        }
    }
}

/// The main error type for the spider framework.
#[derive(Debug, Clone, Error)]
pub enum SpiderError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] ReqwestErrorDetails),
    #[error("Url parsing error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Json parsing error: {0}")]
    JsonError(String),
    #[error("Io error: {0}")]
    IoError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("General error: {0}")]
    GeneralError(String),
    #[error("Failed to convert item to string: {0}")]
    ItemToStringError(String),
    #[error("Error during item serialization: {0}")]
    ItemSerializationError(String),
    #[error("Unknown error")]
    Unknown,
    #[error("Invalid HTTP header value: {0}")]
    InvalidHeaderValue(String),
    #[error("Header value error: {0}")]
    HeaderValueError(String),
    #[error("HTML parsing error: {0}")]
    HtmlParseError(String),
    #[error("UTF-8 parsing error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Pipeline error: {0}")]
    PipelineError(#[from] PipelineError),
    #[error("Request blocked by robots.txt")]
    BlockedByRobotsTxt,
}

impl From<http::header::InvalidHeaderValue> for SpiderError {
    fn from(err: http::header::InvalidHeaderValue) -> Self {
        SpiderError::InvalidHeaderValue(err.to_string())
    }
}

impl From<bincode::Error> for SpiderError {
    fn from(err: bincode::Error) -> Self {
        SpiderError::GeneralError(format!("Bincode error: {}", err))
    }
}

impl From<reqwest::Error> for SpiderError {
    fn from(err: reqwest::Error) -> Self {
        SpiderError::ReqwestError(err.into())
    }
}

impl From<std::io::Error> for SpiderError {
    fn from(err: std::io::Error) -> Self {
        SpiderError::IoError(err.to_string())
    }
}

impl From<SerdeJsonError> for SpiderError {
    fn from(err: SerdeJsonError) -> Self {
        SpiderError::JsonError(err.to_string())
    }
}

/// The error type for item processing pipelines.
#[derive(Error, Debug, Clone)]
pub enum PipelineError {
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Item processing error: {0}")]
    ItemError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("CSV error: {0}")]
    CsvError(String),
    #[error("Other pipeline error: {0}")]
    Other(String),
}

impl From<csv::Error> for PipelineError {
    fn from(err: csv::Error) -> Self {
        PipelineError::CsvError(err.to_string())
    }
}

impl From<std::io::Error> for PipelineError {
    fn from(err: std::io::Error) -> Self {
        PipelineError::IoError(err.to_string())
    }
}

impl From<SerdeJsonError> for PipelineError {
    fn from(err: SerdeJsonError) -> Self {
        PipelineError::SerializationError(err.to_string())
    }
}

impl From<rusqlite::Error> for PipelineError {
    fn from(err: rusqlite::Error) -> Self {
        PipelineError::DatabaseError(err.to_string())
    }
}

impl From<rusqlite::Error> for SpiderError {
    fn from(err: rusqlite::Error) -> Self {
        SpiderError::PipelineError(PipelineError::DatabaseError(err.to_string()))
    }
}
