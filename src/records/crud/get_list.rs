use serde::{Deserialize, de::DeserializeOwned};

use crate::PocketBase;
use crate::error::RequestError;
use crate::{Collection, RecordList};

pub struct CollectionGetListBuilder<'a, T: Send + Deserialize<'a>> {
    client: &'a PocketBase,
    collection_name: &'a str,
    page: Option<String>,
    per_page: Option<String>,
    sort: Option<&'a str>,
    expand: Option<&'a str>,
    filter: Option<&'a str>,
    skip_total: bool,
    _marker: std::marker::PhantomData<T>,
}

impl<'a> Collection<'a> {
    /// Fetch a paginated records list from the given collection.
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
    /// let articles = pb
    ///     .collection("articles")
    ///     .get_list::<Article>()
    ///     .sort("-created,id")
    ///     .call()
    ///     .await?;
    ///
    /// for article in articles.items {
    ///     println!("{article:?}");
    /// }
    /// ```
    #[must_use]
    pub const fn get_list<T: Default + DeserializeOwned + Clone + Send>(
        self,
    ) -> CollectionGetListBuilder<'a, T> {
        CollectionGetListBuilder {
            client: self.client,
            collection_name: self.name,
            page: None,
            per_page: None,
            sort: None,
            expand: None,
            filter: None,
            skip_total: false,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T: Default + DeserializeOwned + Clone + Send> CollectionGetListBuilder<'a, T> {
    /// The page (aka. offset) of the paginated list (default to 1).
    pub fn page(mut self, page: u16) -> Self {
        self.page = Some(page.to_string());
        self
    }

    /// Set the max returned records per page (default: 30, max: 500).
    pub fn per_page(mut self, per_page: u16) -> Self {
        self.per_page = Some(per_page.to_string());
        self
    }

    /// Specify the records order attribute(s).
    /// Add `-`/`+` (default) in front of the attribute for DESC / ASC order.
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

    /// Skip total count query for better performance.
    ///
    /// When enabled, `totalItems` and `totalPages` will be `-1`.
    /// Useful for cursor pagination or when totals aren't needed.
    pub const fn skip_total(mut self, skip_total: bool) -> Self {
        self.skip_total = skip_total;
        self
    }

    /// Execute the request and return the paginated results.
    pub async fn call(self) -> Result<RecordList<T>, RequestError> {
        let url = format!(
            "{}/api/collections/{}/records",
            self.client.base_url, self.collection_name
        );

        let mut query_parameters: Vec<(&str, &str)> = vec![];

        if let Some(page) = self.page.as_deref() {
            query_parameters.push(("page", page));
        }

        if let Some(per_page) = self.per_page.as_deref() {
            query_parameters.push(("perPage", per_page));
        }

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
        let records = response
            .json::<RecordList<T>>()
            .await
            .map_err(|error| RequestError::ParseError(error.to_string()))?;

        Ok(records)
    }
}
