// Copyright (c) Microsoft Corporation
// SPDX-License-Identifier: MIT

use http::{uri::InvalidUri, StatusCode};
use std::error::Error as StdError;
use std::fmt::Display;

#[derive(Debug)]
pub struct Error(Box<ErrorType>);

impl Error {
    fn new(error: ErrorType) -> Self {
        Self(Box::new(error))
    }

    pub fn io(message: String, error: std::io::Error) -> Self {
        Self::new(ErrorType::IO(message, error))
    }

    pub fn hyper(error: HyperErrorType) -> Self {
        Self::new(ErrorType::Hyper(error))
    }

    pub fn hex(message: String, error: hex::FromHexError) -> Self {
        Self::new(ErrorType::Hex(message, error))
    }

    pub fn key(error: KeyErrorType) -> Self {
        Self::new(ErrorType::Key(error))
    }

    pub fn parse_url(url: String, error: InvalidUri) -> Self {
        Self::new(ErrorType::ParseUrl(url, error.to_string()))
    }

    pub fn parse_url_message(url: String, message: String) -> Self {
        Self::new(ErrorType::ParseUrl(url, message))
    }

    pub fn wire_server(error_type: WireServerErrorType, message: String) -> Self {
        Self::new(ErrorType::WireServer(error_type, message))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for Error {}

#[derive(Debug, thiserror::Error)]
enum ErrorType {
    #[error("IO error: {0}: {1}")]
    IO(String, std::io::Error),

    #[error("{0}")]
    Hyper(HyperErrorType),

    #[error("Hex encoded key '{0}' is invalid: {1}")]
    Hex(String, hex::FromHexError),

    #[error("Key error: {0}")]
    Key(KeyErrorType),

    #[error("{0} with the error: {1}")]
    WireServer(WireServerErrorType, String),

    #[error("Failed to parse URL {0} with error: {1}")]
    ParseUrl(String, String),
}

#[derive(Debug, thiserror::Error)]
pub enum HyperErrorType {
    #[error("{0}: {1}")]
    Custom(String, hyper::Error),

    #[error("Failed to build request with error: {0}")]
    RequestBuilder(String),

    #[error("Failed to get response from {0}, status code: {1}")]
    ServerError(String, StatusCode),

    #[error("Deserialization failed: {0}")]
    Deserialize(String),
}

#[derive(Debug, thiserror::Error)]
pub enum WireServerErrorType {
    #[error("Telemetry call to wire server failed")]
    Telemetry,

    #[error("Goal state call to wire server failed")]
    GoalState,

    #[error("Shared config call to wire server failed")]
    SharedConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum KeyErrorType {
    #[error("Key status validation failed with the error: {0}")]
    KeyStatusValidation(String),

    #[error("Failed to send {0} key with error: {1}")]
    SendKeyRequest(String, String),

    #[error("Failed to {0} key with status code: {1}")]
    KeyResponse(String, StatusCode),

    #[error("Failed to join {0} and {1} with error: {2}")]
    ParseKeyUrl(String, String, InvalidUri),
}

#[cfg(test)]
mod test {
    use super::{Error, KeyErrorType, WireServerErrorType};
    use http::StatusCode;

    #[test]
    fn error_formatting_test() {
        let mut error = Error::hyper(super::HyperErrorType::ServerError(
            "testurl.com".to_string(),
            StatusCode::from_u16(500).unwrap(),
        ));
        assert_eq!(
            error.to_string(),
            "Failed to get response from testurl.com, status code: 500 Internal Server Error"
        );

        error = Error::wire_server(
            WireServerErrorType::Telemetry,
            "Invalid response".to_string(),
        );
        assert_eq!(
            error.to_string(),
            "Telemetry call to wire server failed with the error: Invalid response"
        );

        error = Error::key(KeyErrorType::SendKeyRequest(
            "acquire".to_string(),
            error.to_string(),
        ));
        assert_eq!(
            error.to_string(),
            "Key error: Failed to send acquire key with error: Telemetry call to wire server failed with the error: Invalid response"
        );
    }
}
