//! Error and Result types.

use lzma;
use reqwest;
use reqwest::header::ToStrError;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, Error>;

/// The ErrorTypes provide a rough classification of the errors.
#[derive(Debug, Clone)]
pub enum ErrorType {
    DownloadFailure,
    InvalidReleaseFormat,
    UnknownArchitecture,
    VerificationError,
    InvalidDistro,
    UnknownPriority,
    InvalidPackageMeta,
    UnknownVersionRelation,
    InvalidArchitecture,
}

/// Libapt error type.
#[derive(Debug, Clone)]
pub struct Error {
    message: Option<String>,
    error_type: ErrorType,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self.error_type {
            ErrorType::DownloadFailure => "Download failed",
            ErrorType::InvalidReleaseFormat => "Invalid Release file format",
            ErrorType::UnknownArchitecture => "Unknown Architecture",
            ErrorType::VerificationError => "Invalid value",
            ErrorType::InvalidDistro => "Invalid distro",
            ErrorType::UnknownPriority => "Unknown priority",
            ErrorType::InvalidPackageMeta => "Invalid package metadata",
            ErrorType::UnknownVersionRelation => "Unknown package version relation",
            ErrorType::InvalidArchitecture => "Not supported architecture",
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
        Error::from_error(&error, ErrorType::DownloadFailure, &url)
    }

    pub fn from_to_str_error(error: ToStrError, url: &str) -> Error {
        Error::from_error(&error, ErrorType::DownloadFailure, &url)
    }

    pub fn from_lzma(error: lzma::LzmaError, url: &str) -> Error {
        Error::from_error(&error, ErrorType::DownloadFailure, &url)
    }

    pub fn from_io_error(error: io::Error, url: &str) -> Error {
        Error::from_error(&error, ErrorType::DownloadFailure, &url)
    }

    pub fn from_utf8_error(error: FromUtf8Error, url: &str) -> Error {
        Error::from_error(&error, ErrorType::DownloadFailure, &url)
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
