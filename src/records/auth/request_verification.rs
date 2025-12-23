use std::collections::HashMap;

use crate::Collection;
use crate::error::RequestError;

impl<'a> Collection<'a> {
    /// Sends users account verification request.
    ///
    /// # Example
    /// ```rust,ignore
    /// pb.collection("users")
    ///     .request_verification("test@example.com")
    ///     .await?;
    /// ```
    pub async fn request_verification(&self, email: &'a str) -> Result<(), RequestError> {
        let url = format!(
            "{}/api/collections/{}/request-verification",
            self.client.base_url, self.name
        );

        let email: HashMap<String, String> = HashMap::from([("email".to_string(), email.into())]);

        let request = (self.client.request_post_json(&url, &email)).send().await;

        match request {
            Ok(response) => match response.status() {
                reqwest::StatusCode::NO_CONTENT => Ok(()),
                reqwest::StatusCode::BAD_REQUEST => Err(RequestError::BadRequest(String::new())),
                reqwest::StatusCode::NOT_FOUND => Err(RequestError::NotFound),
                _ => Err(RequestError::Unhandled),
            },
            Err(error) => {
                if let Some(error_status) = error.status() {
                    match error_status {
                        reqwest::StatusCode::UNAUTHORIZED => {
                            return Err(RequestError::Unauthorized);
                        }
                        reqwest::StatusCode::FORBIDDEN => {
                            return Err(RequestError::Forbidden);
                        }
                        reqwest::StatusCode::NOT_FOUND => {
                            return Err(RequestError::NotFound);
                        }
                        _ => return Err(RequestError::Unhandled),
                    }
                }

                Err(RequestError::Unhandled)
            }
        }
    }
}
