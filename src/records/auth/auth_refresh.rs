use crate::error::RequestError;
use crate::{AuthStore, Collection};

impl Collection<'_> {
    /// Returns a new auth response (token and record data) for an **already authenticated record**.
    ///
    /// On success, this function returns an `AuthStore` instance with the new token and updated
    /// user information. If an error occurs, it returns a `RequestError`, which may include:
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `RequestError::Unauthorized` if the provided token is invalid.
    /// - `RequestError::Forbidden` if the operation is not permitted.
    /// - `RequestError::NotFound` if the target user or session cannot be located.
    /// - `RequestError::Unhandled` for all other error cases.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::error::Error;
    ///
    /// use pocketbase_rs::PocketBase;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///    let mut pb = PocketBase::new("http://localhost:8090");
    ///
    ///    let auth_data = pb
    ///        .collection("_superusers")
    ///        .auth_with_password("test@test.com", "abcdefghijkl")
    ///        .await?;
    ///
    ///    println!("pre auth data: {auth_data:?}");
    ///
    ///    let auth_data = pb.collection("_superusers").auth_refresh().await?;
    ///
    ///    println!("post auth data: {auth_data:?}");
    ///
    ///    Ok(())
    ///}
    ///
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
