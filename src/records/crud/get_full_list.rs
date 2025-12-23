use serde::de::DeserializeOwned;

use crate::error::RequestError;
use crate::{Collection, RecordList};

/// Builder for fetching all records from a collection.
pub struct CollectionGetFullListBuilder<'a, T: Send> {
    client: &'a crate::PocketBase,
    collection_name: &'a str,
    batch_size: u16,
    sort: Option<&'a str>,
    expand: Option<&'a str>,
    filter: Option<&'a str>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a> Collection<'a> {
    /// Fetch all records from the collection.
    ///
    /// Automatically handles pagination by iterating through all pages.
    /// For performance, `skipTotal` is automatically set to `true`.
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
    /// let all_articles = pb
    ///     .collection("articles")
    ///     .get_full_list::<Article>()
    ///     .sort("-created")
    ///     .call()
    ///     .await?;
    ///
    /// println!("Total articles: {}", all_articles.len());
    /// ```
    #[must_use]
    pub const fn get_full_list<T: Default + DeserializeOwned + Clone + Send>(
        self,
    ) -> CollectionGetFullListBuilder<'a, T> {
        CollectionGetFullListBuilder {
            client: self.client,
            collection_name: self.name,
            batch_size: 500, // Maximum allowed by PocketBase
            sort: None,
            expand: None,
            filter: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T: Default + DeserializeOwned + Clone + Send> CollectionGetFullListBuilder<'a, T> {
    /// Set the batch size for pagination (default: 500, max: 500).
    ///
    /// Lower values reduce memory usage but increase request count.
    pub fn batch_size(mut self, size: u16) -> Self {
        self.batch_size = size.min(500); // Ensure we don't exceed PocketBase's limit
        self
    }

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

    /// Execute the request and return all matching records.
    ///
    /// Automatically handles pagination by making multiple requests if needed.
    pub async fn call(self) -> Result<Vec<T>, RequestError> {
        let mut all_records = Vec::new();
        let mut page = 1u32;
        let batch_size_str = self.batch_size.to_string();

        loop {
            let url = format!(
                "{}/api/collections/{}/records",
                self.client.base_url, self.collection_name
            );

            let page_str = page.to_string();
            let mut query_parameters: Vec<(&str, &str)> = vec![
                ("page", &page_str),
                ("perPage", &batch_size_str),
                ("skipTotal", "true"),
            ];

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
                        Some(reqwest::StatusCode::UNAUTHORIZED) => RequestError::Unauthorized,
                        _ => RequestError::Unhandled,
                    })?,
                Err(error) => {
                    return Err(if error.is_timeout() || error.is_connect() {
                        RequestError::Unreachable
                    } else {
                        match error.status() {
                            Some(reqwest::StatusCode::FORBIDDEN) => RequestError::Forbidden,
                            Some(reqwest::StatusCode::NOT_FOUND) => RequestError::NotFound,
                            Some(reqwest::StatusCode::UNAUTHORIZED) => RequestError::Unauthorized,
                            _ => RequestError::Unhandled,
                        }
                    });
                }
            };

            // Parse JSON response
            let records_page = response
                .json::<RecordList<T>>()
                .await
                .map_err(|error| RequestError::ParseError(error.to_string()))?;

            let items_count = records_page.items.len();
            all_records.extend(records_page.items);

            // Check if we've fetched all records
            // Since we're using skipTotal=true, we can't rely on total_pages
            // Instead, we check if we got fewer items than requested
            if items_count < self.batch_size as usize {
                break;
            }

            page += 1;
        }

        Ok(all_records)
    }
}
