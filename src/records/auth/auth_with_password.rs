use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::{AuthStore, Collection, ErrorResponse};

#[derive(Clone, Default, Serialize)]
struct Credentials<'a> {
    pub(crate) identity: &'a str,
    pub(crate) password: &'a str,
}

/// Represents errors that can occur during the authentication process with the `PocketBase` API.
///
/// This enum defines various error types that may arise when attempting to authenticate,
/// each providing details about the specific issue encountered.
#[derive(Error, Debug)]
pub enum AuthenticationError {
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [400 Bad Request]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400") HTTP error response.
    ///
    /// Tip: The credentials you provided may be incorrect.
    #[error("Authentication failed: Invalid Credentials. Given email and/or password is wrong.")]
    InvalidCredentials,
    /// Email and/or Password cannot be empty.
    ///
    /// This variant indicates that certain fields in the authentication request need to be validated.
    /// The fields are represented as booleans:
    ///
    /// - `identity`: is blank and shouldn't be.
    /// - `password`: is blank and shouldn't be.
    #[error(
        "Authentication failed: Empty Credential Field. Given email and/or password is empty."
    )]
    EmptyField {
        /// Is identity blank.
        identity: bool,
        /// Is password blank.
        password: bool,
    },
    /// The provided identity must be an email address.
    ///
    /// This variant indicates that the authentication request failed because the provided identity
    /// does not conform to the expected email format. The `PocketBase` API requires the identity to
    /// be a valid email address for authentication.
    #[error("Authentication failed. Given identity is not a valid email.")]
    IdentityMustBeEmail,
    /// An HTTP error occurred while communicating with the `PocketBase` API.
    ///
    /// This variant wraps a [`reqwest::Error`] and indicates that the request could not be completed
    /// due to network issues, invalid URL, timeouts, etc.
    #[error("Authentication failed. Couldn't reach the PocketBase API: {0}")]
    HttpError(reqwest::Error),
    /// When something unexpected was returned by the `PocketBase` REST API.
    ///
    /// Would usually mean that there is an error somewhere in this API wrapper.
    #[error("Authentication failed due to an unexpected response. Usually means a problem in the PocketBase API's wrapper.")]
    UnexpectedResponse,
    /// Occurs when you try to authenticate a `PocketBase` client without providing the collection name.
    #[error("Authentication failed due to missing collection name. [Example: PocketBaseClientBuilder::new(\"\")")]
    MissingCollection,
}

impl From<reqwest::Error> for AuthenticationError {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpError(error)
    }
}

impl Collection<'_> {
    /// Authenticates a Client user with the `PocketBase` server using their email and password.
    ///
    /// This method performs password-based authentication against the specified collection.
    /// Upon successful authentication, the client's internal auth store is updated with the
    /// authentication token and user information, which will be automatically included in
    /// subsequent API requests.
    ///
    /// # Parameters
    ///
    /// * `identity`: The **username** or **email** of the Client record to authenticate.
    /// * `password`: The auth record password.
    ///
    /// # Returns
    ///
    /// Returns `Ok(AuthStore)` containing:
    /// - The authentication token for API requests
    /// - The authenticated user's record information
    ///
    /// Returns `Err(AuthenticationError)` for various failure cases.
    ///
    /// # Errors
    ///
    /// This function will return an `AuthenticationError` if:
    ///
    /// - `InvalidCredentials`: The provided email/password combination is incorrect
    /// - `EmptyField`: Either the identity or password field is empty
    /// - `IdentityMustBeEmail`: The identity field doesn't contain a valid email format
    /// - `HttpError`: Network or connection issues occurred
    /// - `UnexpectedResponse`: The server response was not in the expected format
    /// - `MissingCollection`: No collection name was provided
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::error::Error;
    /// use pocketbase_rs::PocketBase;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut pb = PocketBase::new("http://localhost:8090");
    ///
    ///     // Authenticate with a users collection
    ///     let auth_data = pb.collection("users")
    ///         .auth_with_password("test@domain.com", "secure-password")
    ///         .await?;
    ///
    ///     println!("Authenticated as: {}", auth_data.record.email);
    ///     println!("Token: {}", auth_data.token);
    ///
    ///     // The token is now automatically included in future requests
    ///     let profile = pb.collection("profiles")
    ///         .get_one::<Profile>("some_id")
    ///         .call()
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn auth_with_password(
        &mut self,
        identity: &str,
        password: &str,
    ) -> Result<AuthStore, AuthenticationError> {
        let uri = format!(
            "{}/api/collections/{}/auth-with-password",
            self.client.base_url, self.name
        );

        let credentials = Credentials { identity, password };

        let response = self
            .client
            .request_post_json(&uri, &credentials)
            .send()
            .await?;

        if response.status().is_success() {
            let auth_store = response.json::<AuthStore>().await?;

            self.client.update_auth_store(auth_store.clone());

            return Ok(auth_store);
        }

        if response.status() == reqwest::StatusCode::BAD_REQUEST {
            let error_response: ErrorResponse =
                response.json().await.unwrap_or_else(|_| ErrorResponse {
                    code: 400,
                    message: "Unknown error".to_string(),
                    data: None,
                });

            if let Some(ref data) = error_response.data {
                // {
                //     "code": 400,
                //     "message": "Failed to authenticate.",
                //     "data": {}
                // }
                if data.as_object().is_some_and(serde_json::Map::is_empty) {
                    return Err(AuthenticationError::InvalidCredentials);
                }

                // Check for specific field validation errors
                let identity_error = data
                    .get("identity")
                    .and_then(|v| v.get("code").and_then(Value::as_str));

                match identity_error {
                    // {
                    //     "code": 400,
                    //     "message": "Something went wrong while processing your request.",
                    //     "data": {
                    //       "identity": {
                    //         "code": "validation_is_email",
                    //         "message": "Must be a valid email address."
                    //       }
                    //     }
                    // }
                    Some("validation_is_email") => {
                        return Err(AuthenticationError::IdentityMustBeEmail)
                    }

                    // {
                    //     "code": 400,
                    //     "message": "Something went wrong while processing your request.",
                    //     "data": {
                    //       "identity": {
                    //         "code": "validation_required",
                    //         "message": "Cannot be blank."
                    //       },
                    //       "password": {
                    //         "code": "validation_required",
                    //         "message": "Cannot be blank."
                    //       }
                    //     }
                    // }
                    Some("validation_required") => {
                        return Err(AuthenticationError::EmptyField {
                            identity: identity_error.is_some(),
                            password: data.get("password").is_some(),
                        })
                    }
                    None => {
                        let password_error = data.get("password").is_some();
                        return Err(AuthenticationError::EmptyField {
                            identity: false,
                            password: password_error,
                        });
                    }
                    _ => {}
                }
            }

            // {
            //     "code": 400,
            //     "message": "Failed to authenticate.",
            //     "data": {}
            // }
            return Err(AuthenticationError::InvalidCredentials);
        }

        Err(AuthenticationError::UnexpectedResponse)
    }
}
