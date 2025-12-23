use crate::Collection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeleteError {
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [400 Bad Request]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400") HTTP error response.
    ///
    /// Failed to delete record. Make sure that the record is not part of a required relation reference. `PocketBase`.
    #[error(
        "Failed to delete record. Make sure that the record is not part of a required relation reference."
    )]
    BadRequest,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [403 Forbidden]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403") HTTP error response.
    ///
    /// You are not allowed to perform this request.
    #[error("You are not allowed to perform this request.")]
    Forbidden,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [404 Not Found]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404") HTTP error response.
    ///
    /// The requested resource wasn't found.
    #[error("The requested resource wasn't found.")]
    NotFound,
    /// Communication with the `PocketBase` API failed.
    ///
    /// This could be caused by an internet outage, an error in the link given to the `PocketBase` SDK
    /// and similar errors.
    #[error("The communication with the PocketBase API failed: {0}")]
    Unreachable(String),
    /// An unexpected error occurred.
    /// The response from the `PocketBase` instance API was unexpected.
    /// If you think its an error, please [open an issue on GitHub]("https://github.com/fromhorizons/pocketbase-rs/issues").
    #[error("An unhandled status code was returned by the PocketBase API: {0}")]
    UnexpectedResponse(String),
}

impl<'a> Collection<'a> {
    /// Delete a single record.
    ///
    /// # Example
    /// ```rust,ignore
    /// pb.collection("articles")
    ///     .delete("RECORD_ID")
    ///     .await?;
    /// ```
    pub async fn delete(&self, record_id: &'a str) -> Result<(), DeleteError> {
        // Validate record_id
        if record_id.is_empty() {
            return Err(DeleteError::BadRequest);
        }

        let endpoint = format!(
            "{}/api/collections/{}/records/{}",
            self.client.base_url, self.name, record_id
        );
        let request = self.client.request_delete(&endpoint).send().await;

        match request {
            Ok(response) => match response.status() {
                reqwest::StatusCode::NO_CONTENT | reqwest::StatusCode::OK => Ok(()),
                reqwest::StatusCode::BAD_REQUEST => Err(DeleteError::BadRequest),
                reqwest::StatusCode::FORBIDDEN => Err(DeleteError::Forbidden),
                reqwest::StatusCode::NOT_FOUND => Err(DeleteError::NotFound),
                _ => Err(DeleteError::UnexpectedResponse(format!(
                    "Status: {}, Collection: {}, Record: {}",
                    response.status(),
                    self.name,
                    record_id
                ))),
            },
            Err(e) => {
                if e.is_timeout() {
                    Err(DeleteError::Unreachable("Request timed out".to_string()))
                } else if e.is_connect() {
                    Err(DeleteError::Unreachable(
                        "Failed to connect to server".to_string(),
                    ))
                } else {
                    Err(DeleteError::Unreachable(e.to_string()))
                }
            }
        }
    }
}
