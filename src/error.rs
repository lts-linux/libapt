//! Error and Result types.
use serde::{Deserialize, Serialize};

use lzma;
use reqwest;
use reqwest::header::ToStrError;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, Error>;

/// The ErrorTypes provide a rough classification of the errors.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum ErrorType {
    Download,
    InReleaseFormat,
    InReleaseStandard,
    UnknownArchitecture,
    Verification,
    DistroFormat,
    UnknownPriority,
    PackageFormat,
    SourceFormat,
    UnknownVersionRelation,
    InvalidArchitecture,
    InvalidReference,
    ApiUsage,
    Version,
}

/// Libapt error type.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Error {
    message: Option<String>,
    error_type: ErrorType,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.error_type {
            ErrorType::Download => "Download failed",
            ErrorType::InReleaseFormat => "Invalid Release file format",
            ErrorType::UnknownArchitecture => "Unknown Architecture",
            ErrorType::Verification => "Invalid value",
            ErrorType::DistroFormat => "Invalid distro",
            ErrorType::UnknownPriority => "Unknown priority",
            ErrorType::PackageFormat => "Invalid package metadata",
            ErrorType::SourceFormat => "Invalid source package metadata",
            ErrorType::UnknownVersionRelation => "Unknown package version relation",
            ErrorType::InvalidArchitecture => "Not supported architecture",
            ErrorType::InvalidReference => "Invalid URL reference",
            ErrorType::ApiUsage => "API usage issue",
            ErrorType::InReleaseStandard => "Debian policy InRelease standard violation",
            ErrorType::Version => "Invalid package version",
        };

        if let Some(message) = &self.message {
            write!(f, "{}:{}", description, message)
        } else {
            write!(f, "{}", description)
        }
    }
}

impl Error {
    pub fn new(message: &str, error_type: ErrorType) -> Error {
        Error {
            message: Some(message.to_string()),
            error_type: error_type,
        }
    }

    pub fn from_reqwest(error: reqwest::Error, url: &str) -> Error {
        Error::from_error(&error, ErrorType::Download, &url)
    }

    pub fn from_to_str_error(error: ToStrError, url: &str) -> Error {
        Error::from_error(&error, ErrorType::Download, &url)
    }

    pub fn from_lzma(error: lzma::LzmaError, url: &str) -> Error {
        Error::from_error(&error, ErrorType::Download, &url)
    }

    pub fn from_io_error(error: io::Error, url: &str) -> Error {
        Error::from_error(&error, ErrorType::Download, &url)
    }

    pub fn from_utf8_error(error: FromUtf8Error, url: &str) -> Error {
        Error::from_error(&error, ErrorType::Download, &url)
    }

    pub fn from_error(
        error: &dyn std::error::Error,
        error_type: ErrorType,
        message: &str,
    ) -> Error {
        let message = format!("{message}: {error}");
        Error {
            message: Some(message),
            error_type: error_type,
        }
    }

    pub fn from_type(error_type: ErrorType) -> Error {
        Error {
            message: None,
            error_type: error_type,
        }
    }
}
