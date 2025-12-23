use serde::{Deserialize, de::DeserializeOwned};

use crate::error::RequestError;
use crate::{Collection, PocketBase};

pub struct CollectionGetOneBuilder<'a, T: Send + Deserialize<'a>> {
    client: &'a PocketBase,
    collection_name: &'a str,
    record_id: &'a str,
    expand: Option<&'a str>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a> Collection<'a> {
    /// Fetch a single record.
    ///
    /// # Example
    /// ```rust,ignore
    /// #[derive(Default, Deserialize, Clone)]
    /// struct Article {
    ///     id: String,
    ///     title: String,
    ///     content: String,
    /// }
    ///
    /// let article = pb
    ///     .collection("articles")
    ///     .get_one::<Article>("record_id_123")
    ///     .call()
    ///     .await?;
    /// ```
    #[must_use]
    pub const fn get_one<T: Default + DeserializeOwned + Clone + Send>(
        self,
        record_id: &'a str,
    ) -> CollectionGetOneBuilder<'a, T> {
        CollectionGetOneBuilder {
            client: self.client,
            collection_name: self.name,
            record_id,
            expand: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T: Default + DeserializeOwned + Clone + Send> CollectionGetOneBuilder<'a, T> {
    /// Auto expand record relations (up to 6-levels deep).
    ///
    /// Expanded relations are appended under the `expand` property.
    /// Only relations the user has view permissions for will be expanded.
    ///
    /// # Example
    /// ```rust,ignore
    /// .expand("author")
    /// ```
    pub const fn expand(mut self, expand: &'a str) -> Self {
        self.expand = Some(expand);
        self
    }

    /// Execute the request and return the record.
    pub async fn call(self) -> Result<T, RequestError> {
        let url = format!(
            "{}/api/collections/{}/records/{}",
            self.client.base_url, self.collection_name, self.record_id
        );

        let request = self.expand.map_or_else(
            || self.client.request_get(&url, None),
            |expand_value| {
                let expand_params = vec![("expand", expand_value)];

                self.client.request_get(&url, Some(expand_params))
            },
        );

        let request = request.send().await;

        let response = match request {
            Ok(response) => response
                .error_for_status()
                .map_err(|err| match err.status() {
                    Some(reqwest::StatusCode::FORBIDDEN) => RequestError::Forbidden,
                    Some(reqwest::StatusCode::NOT_FOUND) => RequestError::NotFound,
                    Some(reqwest::StatusCode::TOO_MANY_REQUESTS) => RequestError::TooManyRequests,
                    _ => RequestError::Unhandled,
                })?,
            Err(error) => {
                return Err(match error.status() {
                    Some(reqwest::StatusCode::FORBIDDEN) => RequestError::Forbidden,
                    Some(reqwest::StatusCode::NOT_FOUND) => RequestError::NotFound,
                    Some(reqwest::StatusCode::TOO_MANY_REQUESTS) => RequestError::TooManyRequests,
                    _ => RequestError::Unhandled,
                });
            }
        };

        // Parse JSON response
        let record = response
            .json::<T>()
            .await
            .map_err(|error| RequestError::ParseError(error.to_string()))?;

        Ok(record)
    }
}
