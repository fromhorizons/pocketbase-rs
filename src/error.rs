//! Various errors module.

use core::fmt;
use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

pub use crate::records::auth::auth_with_password::AuthenticationError;
pub use crate::records::auth::impersonate::ImpersonateError;
pub use crate::records::crud::create::CreateError;
pub use crate::records::crud::update::UpdateError;

/// This error represents the error returned by the `PocketBase`
/// instance in case of a 400 error.
#[derive(Deserialize, Debug)]
pub struct BadRequestResponse {
    /// HTTP Status Code.
    pub status: u16,
    /// Description from given by `PocketBase` about why the error happened.
    pub message: String,
    /// A list of fields that caused the error.
    pub data: HashMap<String, BadRequestField>,
}

/// Represents an instance of one of the errors that could be returned on a bad request.
///
/// This struct holds detailed information about a single validation error,
/// including the field name, error code, and a user-friendly message.
#[derive(Deserialize, Debug)]
pub struct BadRequestError {
    /// Name of the field.
    pub name: String,
    /// Error code.
    pub code: String,
    /// More details about the error.
    pub message: String,
}

impl fmt::Display for BadRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {}", self.name, self.code, self.message)
    }
}

/// Represents one of the fields that caused the Bad Request error.
#[derive(Deserialize, Debug)]
pub struct BadRequestField {
    /// Error code *(example: `validation_required`)*.
    pub code: String,
    /// A text explaining in a readable way what this error is.
    pub message: String,
}

/// Represents errors when interacting with the `PocketBase` API.
///
/// This enum provides a set of error types that may occur during
/// API requests, each indicating a specific issue encountered.
#[derive(Error, Debug)]
pub enum RequestError {
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [400 Bad Request]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400") HTTP error response.
    ///
    /// Your request may be missing fields or its content doesn't match what `PocketBase` expects to receive.
    #[error("Bad Request: Something went wrong while processing your request. {0}")]
    BadRequest(String),
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [401 Unauthorized]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401") HTTP error response.
    ///
    /// The request may require an Authorization Token.
    #[error("Unauthorized: The request may require an Authorization Token.")]
    Unauthorized,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [403 Forbidden]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403") HTTP error response.
    ///
    /// The authenticated user may not have permissions for this interaction.
    #[error("Forbidden: The authenticated user may not have permissions for this interaction.")]
    Forbidden,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [404 Not Found]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404") HTTP error response.
    #[error("Not Found: The requested resource could not be found.")]
    NotFound,
    /// The response could not be parsed into the expected data structure.
    #[error("Parse Error: Could not parse response into the expected data structure. It usually means that there is a missmatch between the provided Generic Type Parameter and your Collection definition. - {0}")]
    ParseError(String),
    /// The `PocketBase` API interaction timed out. It may be offline.
    #[error(
        "Unreachable: The PocketBase API interaction timed out, or the service may be offline."
    )]
    Unreachable,
    /// Too many requests were sent to the API.
    ///
    /// The server is rate limiting requests. Wait before retrying.
    #[error(
        "Too Many Requests: The server is rate limiting requests. Please wait before retrying."
    )]
    TooManyRequests,
    /// Unhandled error.
    ///
    /// Usually emitted when something unexpected happened, and isn't handled correctly by this crate.
    #[error("Unhandled Error: An unexpected error occurred.")]
    Unhandled,
}
