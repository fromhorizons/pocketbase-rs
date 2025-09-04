use serde::{de::DeserializeOwned, Deserialize};

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
    /// Fetch a single record from the given collection.
    ///
    /// This function returns a `CollectionGetListBuilder`, which allows you to specify
    /// additional options such as filtering or expanding linked records before calling `.call().await` to
    /// execute the request.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::error::Error;
    ///
    /// use pocketbase_rs::PocketBase;
    /// use serde::Deserialize;
    ///
    /// #[derive(Default, Deserialize, Clone)]
    /// struct Article {
    ///     title: String,
    ///     content: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut pb = PocketBase::new("http://localhost:8090");
    ///
    ///     // ...
    ///
    ///     let article = pb
    ///         .collection("articles")
    ///         .get_one::<Article>("record_id_123")
    ///         .call()
    ///         .await?;
    ///
    ///     println!("Article: {article:?}");
    ///
    ///     Ok(())
    /// }
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
    /// Auto expand record relations.
    ///
    /// Example:
    /// ```toml
    /// ?expand=relField1,relField2.subRelField
    /// ```
    ///
    /// Supports up to 6-levels depth nested relations expansion.
    /// The expanded relations will be appended to each individual record under the `expand` property (eg. `"expand": {"relField1": {...}, ...}`).
    /// Only the relations to which the request user has permissions to **view** will be expanded.
    pub const fn expand(mut self, expand: &'a str) -> Self {
        self.expand = Some(expand);
        self
    }

    /// Sends the request and returns the response.
    ///
    /// This method finalizes the request built using the builder pattern
    /// and sends it to the API endpoint. It should be called after all
    /// desired parameters and configurations have been set on the builder.
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
