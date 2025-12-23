use serde::{Deserialize, de::DeserializeOwned};

use crate::PocketBase;
use crate::error::RequestError;
use crate::{Collection, RecordList};

pub struct CollectionGetFirstListItemBuilder<'a, T: Send + Deserialize<'a>> {
    client: &'a PocketBase,
    collection_name: &'a str,
    sort: Option<&'a str>,
    expand: Option<&'a str>,
    filter: Option<&'a str>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a> Collection<'a> {
    /// Fetch the first record from the given collection.
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
    ///     .get_first_list_item::<Article>()
    ///     .sort("-created,id")
    ///     .filter("language='en'")
    ///     .call()
    ///     .await?;
    /// ```
    #[must_use]
    pub const fn get_first_list_item<T: Default + DeserializeOwned + Clone + Send>(
        self,
    ) -> CollectionGetFirstListItemBuilder<'a, T> {
        CollectionGetFirstListItemBuilder {
            client: self.client,
            collection_name: self.name,
            sort: None,
            expand: None,
            filter: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T: Default + DeserializeOwned + Clone + Send> CollectionGetFirstListItemBuilder<'a, T> {
    /// Set the sort order. Prefix with `-` for DESC or `+` for ASC (default).
    ///
    /// # Example
    /// ```rust,ignore
    /// .sort("-created,id") // DESC by created, ASC by id
    /// ```
    pub const fn sort(mut self, sort: &'a str) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Filter the returned records.
    ///
    /// Supports operators: `=`, `!=`, `>`, `>=`, `<`, `<=`, `~`, `!~`
    /// and their "any/at least one" variants with `?` prefix.
    /// Combine with `&&` (AND), `||` (OR), and `(...)` for grouping.
    ///
    /// # Example
    /// ```rust,ignore
    /// .filter("language='en' && created>'1970-01-01'")
    /// ```
    pub const fn filter(mut self, filter: &'a str) -> Self {
        self.filter = Some(filter);
        self
    }

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

    /// Execute the request and return the first matching record.
    pub async fn call(self) -> Result<T, RequestError> {
        let url = format!(
            "{}/api/collections/{}/records",
            self.client.base_url, self.collection_name
        );

        let mut query_parameters: Vec<(&str, &str)> =
            vec![("page", "1"), ("perPage", "1"), ("skipTotal", "true")];

        if let Some(sort) = self.sort {
            query_parameters.push(("sort", sort));
        }

        if let Some(filter) = self.filter {
            query_parameters.push(("filter", filter));
        }

        if let Some(expand) = self.expand {
            query_parameters.push(("expand", expand));
        }

        let request = self
            .client
            .request_get(&url, Some(query_parameters))
            .send()
            .await;

        let response = match request {
            Ok(response) => response
                .error_for_status()
                .map_err(|err| match err.status() {
                    Some(reqwest::StatusCode::FORBIDDEN) => RequestError::Forbidden,
                    Some(reqwest::StatusCode::NOT_FOUND) => RequestError::NotFound,
                    _ => RequestError::Unhandled,
                })?,
            Err(error) => {
                return Err(match error.status() {
                    Some(reqwest::StatusCode::FORBIDDEN) => RequestError::Forbidden,
                    Some(reqwest::StatusCode::NOT_FOUND) => RequestError::NotFound,
                    _ => RequestError::Unhandled,
                });
            }
        };

        // Parse JSON response
        let records = response
            .json::<RecordList<T>>()
            .await
            .map_err(|error| RequestError::ParseError(error.to_string()))?;

        records.items.first().map_or_else(
            || Err(RequestError::ParseError("No record found.".to_owned())),
            |record| Ok(record.clone()),
        )
    }
}
