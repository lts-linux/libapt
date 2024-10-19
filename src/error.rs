use reqwest;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum ErrorType {
    DownloadFailure,
    InvalidReleaseFormat,
    UnknownArchitecture,
    VerificationError,
    InvalidDistro,
}

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

    pub fn from_reqwest(error: reqwest::Error) -> Error {
        Error {
            message: Some(error.to_string()),
            error_type: ErrorType::DownloadFailure,
        }
    }

    pub fn from_type(error_type: ErrorType) -> Error {
        Error {
            message: None,
            error_type: error_type,
        }
    }
}
