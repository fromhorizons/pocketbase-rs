use crate::Collection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeleteError {
    #[error("Failed to delete record. Make sure that the record is not part of a required relation reference.")]
    BadRequest,
    #[error("Only superusers can access this action.")]
    Forbidden,
    #[error("The requested resource wasn't found.")]
    NotFound,
    #[error("The communication with the PocketBase API failed: {0}")]
    Unreachable(String),
    #[error("An unhandled status code was returned by the PocketBase API: {0}")]
    UnexpectedResponse(String),
}

impl<'a> Collection<'a> {
    /// Delete a single record.
    ///
    /// # Arguments
    ///
    /// * `record_id` - ID of the record to delete.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` which is:
    /// * `Ok(())` if the record was successfully deleted.
    /// * `Err(DeleteError)` if there was an error during the deletion process.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The server responds with a bad request status (`DeleteError::BadRequest`).
    /// * The server responds with a forbidden status (`DeleteError::Forbidden`).
    /// * The record is not found (`DeleteError::NotFound`).
    /// * The request to the server fails (`DeleteError::Unreachable`).
    /// * The server responds with an unexpected status (`DeleteError::UnexpectedResponse`).
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
