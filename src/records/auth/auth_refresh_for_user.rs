use crate::error::RequestError;
use crate::{AuthStore, Collection};

impl<'a> Collection<'a> {
    /// Refreshes the auth token for a specific external user.
    ///
    /// This is useful when managing tokens for users other than the currently
    /// authenticated user *(e.g., as a superuser)*.
    /// On success, this function returns an `AuthStore` instance with the new token and updated
    /// user information for the specified user.
    ///
    ///
    /// # Arguments
    ///
    /// - `user_token`: A string slice representing the token of the user whose auth should be refreshed.
    ///
    /// # Errors
    ///
    /// This function may return:
    /// - `RequestError::Unauthorized` if the provided user token is invalid.
    /// - `RequestError::Forbidden` if the operation is not permitted for the provided user.
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
    ///    // ...
    ///
    ///    let auth_data = pb
    ///        .collection("users")
    ///        .auth_refresh_for_user("USER_TOKEN")
    ///        .await?;
    ///
    ///    println!("Auth Data: {auth_data:?}");
    ///
    ///    Ok(())
    ///}
    ///
    /// ```
    pub async fn auth_refresh_for_user(
        &mut self,
        user_token: &'a str,
    ) -> Result<AuthStore, RequestError> {
        let url = format!(
            "{}/api/collections/{}/auth-refresh",
            self.client.base_url(),
            self.name
        );

        // Usually we would do `let request = self.client.request_post(&url).bearer_auth(user_token).send().await;`,
        // but in our wrapper methods around `Reqwest`, we already use the `.bearer_auth()` method on our
        // `RequestBuilder` with the token of the currently logged in user.
        // When we try to reuse `.bearer_auth()` for a second time, for example here to put the **Token** of
        // the user to re-authenticate, it seems to be ignored. We could probably rewrite our wrapper methods, but honestly, I'm too lazy.
        let request = self
            .client
            .reqwest_client
            .post(&url)
            .bearer_auth(user_token)
            .send()
            .await;

        match request {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => {
                    let Ok(auth_store) = response.json::<AuthStore>().await else {
                        return Err(RequestError::Unhandled);
                    };

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
