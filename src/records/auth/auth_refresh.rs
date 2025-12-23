use crate::error::RequestError;
use crate::{AuthStore, Collection};

impl Collection<'_> {
    /// Returns a new auth response (token and record data) for an **already authenticated record**.
    ///
    /// This method is usually called by users on page/screen reload to ensure that the previously stored data in `pb.auth_store()` is still valid and up-to-date.
    ///
    /// # Example
    /// ```rust,ignore
    /// let auth_data = pb.collection("users")
    ///     .auth_refresh()
    ///     .await?;
    ///
    /// println!("New token: {}", auth_data.token);
    /// ```
    pub async fn auth_refresh(&mut self) -> Result<AuthStore, RequestError> {
        let url = format!(
            "{}/api/collections/{}/auth-refresh",
            self.client.base_url(),
            self.name
        );

        let request = self.client.request_post(&url).send().await;

        match request {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => {
                    let Ok(auth_store) = response.json::<AuthStore>().await else {
                        return Err(RequestError::Unhandled);
                    };

                    self.client.update_auth_store(auth_store.clone());

                    Ok(auth_store)
                }

                reqwest::StatusCode::UNAUTHORIZED => Err(RequestError::Unauthorized),
                reqwest::StatusCode::FORBIDDEN => Err(RequestError::Forbidden),
                reqwest::StatusCode::NOT_FOUND => Err(RequestError::NotFound),

                _ => Err(RequestError::Unhandled),
            },
            Err(_) => Err(RequestError::Unhandled),
        }
    }
}
