use serde::Deserialize;
use thiserror::Error;

use super::AuthStore;
use crate::{Collection, PocketBase};

/// Represents the various errors that can be obtained after a `impersonate` request.
#[derive(Error, Debug)]
pub enum ImpersonateError {
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [400 Bad Request]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400") HTTP error response.
    ///
    /// The request requires valid record authorization token to be set.
    #[error("Bad Request: The request requires valid record authorization token to be set.")]
    BadRequest,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [401 Unauthorized]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401") HTTP error response.
    ///
    /// The request requires valid record authorization token.
    #[error("The request requires valid record authorization token.")]
    Unauthorized,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [403 Forbidden]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403") HTTP error response.
    ///
    /// The authorized record is not allowed to perform this action.
    /// Are you impersonating a user from a non-superuser account?
    #[error(
        "The authorized record is not allowed to perform this action. Are you impersonating a user from a non-superuser account?"
    )]
    Forbidden,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [404 Not Found]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404") HTTP error response.
    ///
    /// The requested resource wasn't found.
    /// The given user id is probably wrong.
    #[error("The requested resource wasn't found.")]
    NotFound,
    /// Communication with the `PocketBase` API failed.
    ///
    /// This could be caused by an internet outage, an error in the link given to the `PocketBase` SDK
    /// and similar errors.
    #[error("The communication with the PocketBase API failed: {0}")]
    Unreachable(String),
    /// The response from the `PocketBase` instance API was unexpected.
    /// If you think its an error, please [open an issue on GitHub]("https://github.com/fromhorizons/pocketbase-rs/issues").
    #[error("An unhandled status code was returned by the PocketBase API: {0}")]
    UnexpectedResponse(String),
}

#[derive(Deserialize)]
struct AuthData {
    record: AuthDataRecord,
    token: String,
}

#[derive(Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthDataRecord {
    collection_id: String,
    collection_name: String,
    id: String,
    email: String,
    email_visibility: bool,
    verified: bool,
    created: String,
    updated: String,
}

pub struct CollectionImpersonateBuilder<'a> {
    client: &'a PocketBase,
    collection_name: &'a str,
    user_id: &'a str,
    duration: Option<String>,
}

impl<'a> Collection<'a> {
    /// Authenticate as a different user by generating a non-refreshable auth token.
    ///
    /// Only superusers can perform this action. Returns a new `PocketBase` client
    /// with the impersonated user's auth token.
    ///
    /// # Example
    /// ```rust,ignore
    /// let impersonate_client = pb
    ///     .collection("users")
    ///     .impersonate("USER_RECORD_ID")
    ///     .duration(3600)
    ///     .call()
    ///     .await?;
    ///
    /// println!("Token: {}", impersonate_client.auth_store().unwrap().token);
    /// ```
    #[must_use]
    pub const fn impersonate(self, user_id: &'a str) -> CollectionImpersonateBuilder<'a> {
        CollectionImpersonateBuilder {
            client: self.client,
            collection_name: self.name,
            user_id,
            duration: None,
        }
    }
}

impl CollectionImpersonateBuilder<'_> {
    /// Set custom JWT duration in seconds (optional).
    ///
    /// If not set, uses the default collection auth token duration.
    pub fn duration(mut self, duration: u128) -> Self {
        self.duration = Some(duration.to_string());
        self
    }

    /// Execute the request and return a new `PocketBase` client with the impersonated user's token.
    pub async fn call(self) -> Result<PocketBase, ImpersonateError> {
        let url = format!(
            "{}/api/collections/{}/impersonate/{}",
            self.client.base_url, self.collection_name, self.user_id
        );

        let request = {
            if let Some(duration) = self.duration {
                self.client
                    .request_post_form(
                        &url,
                        reqwest::multipart::Form::new().text("duration", duration),
                    )
                    .send()
                    .await
            } else {
                self.client.request_post(&url).send().await
            }
        };

        match request {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => {
                    let Ok(auth_store) = response.json::<AuthStore>().await else {
                        return Err(ImpersonateError::UnexpectedResponse(
                            "Couldn't parse API response into Auth Data".to_string(),
                        ));
                    };

                    let mut impersonate_client = PocketBase::new(&self.client.base_url());
                    impersonate_client.update_auth_store(auth_store);

                    Ok(impersonate_client)
                }

                reqwest::StatusCode::BAD_REQUEST => Err(ImpersonateError::BadRequest),
                reqwest::StatusCode::UNAUTHORIZED => Err(ImpersonateError::Unauthorized),
                reqwest::StatusCode::FORBIDDEN => Err(ImpersonateError::Forbidden),
                reqwest::StatusCode::NOT_FOUND => Err(ImpersonateError::NotFound),

                _ => Err(ImpersonateError::UnexpectedResponse(
                    response.status().to_string(),
                )),
            },
            Err(error) => Err(ImpersonateError::Unreachable(error.to_string())),
        }
    }
}
