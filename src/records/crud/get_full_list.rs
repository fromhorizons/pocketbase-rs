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
    /// Fetch all records from the collection by iterating through all pages.
    ///
    /// This method automatically handles pagination and returns all records
    /// from the collection that match the specified filters. For performance
    /// reasons, the `skipTotal` parameter is automatically set to `true`.
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
    ///     id: String,
    ///     title: String,
    ///     content: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut pb = PocketBase::new("http://localhost:8090");
    ///
    ///     // Authenticate first
    ///     pb.collection("users")
    ///         .auth_with_password("user@example.com", "password")
    ///         .await?;
    ///
    ///     // Get all articles
    ///     let all_articles = pb
    ///         .collection("articles")
    ///         .get_full_list::<Article>()
    ///         .sort("-created")
    ///         .call()
    ///         .await?;
    ///
    ///     println!("Total articles: {}", all_articles.len());
    ///     for article in all_articles {
    ///         println!("Article: {:?}", article);
    ///     }
    ///
    ///     Ok(())
    /// }
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
    /// Lower values may result in more requests but can be useful
    /// for reducing memory usage when dealing with large records.
    pub fn batch_size(mut self, size: u16) -> Self {
        self.batch_size = size.min(500); // Ensure we don't exceed PocketBase's limit
        self
    }

    /// Specify the records order attribute(s).
    /// Add `-`/`+` (default) in front of the attribute for DESC / ASC order.
    ///
    /// Example:
    /// ```toml
    /// ?sort=-created,id # DESC by created and ASC by id
    /// ```
    pub const fn sort(mut self, sort: &'a str) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Filter the returned records.
    ///
    /// Example:
    /// ```toml
    /// ?filter=(id="abc" && created>'1970-01-01')
    /// ```
    ///
    /// The syntax basically follows the format
    /// `OPERAND OPERATOR OPERAND`, where:
    /// - `OPERAND` - could be any of the above field literal, string (single or double quoted), number, null, true, false
    /// - `OPERATOR` - is one of:
    ///    - `=`     Equal
    ///    - `!=`   NOT equal
    ///    - `>`     Greater than
    ///    - `>=`   Greater than or equal
    ///    - `<`     Less than
    ///    - `<=`   Less than or equal
    ///    - `~`     Like/Contains (if not specified auto wraps the right string OPERAND in a "%" for wildcard match)
    ///    - `!~`   NOT Like/Contains (if not specified auto wraps the right string OPERAND in a "%" for wildcard match)
    ///    - `?=`    *Any/At least one of* Equal
    ///    - `?!=`  *Any/At least one of* NOT equal
    ///    - `?>`    *Any/At least one of* Greater than
    ///    - `?>=`  *Any/At least one of* Greater than or equal
    ///    - `?<`    *Any/At least one of* Less than
    ///    - `?<=`  *Any/At least one of* Less than or equal
    ///    - `?~`    *Any/At least one of* Like/Contains (if not specified auto wraps the right string OPERAND in a "%" for wildcard match)
    ///    - `?!~` *Any/At least one of* NOT Like/Contains (if not specified auto wraps the right string OPERAND in a "%" for wildcard match)
    ///
    /// To group and combine several expressions you could use brackets `(...)`, `&&` (AND) and `||` (OR) tokens.
    pub const fn filter(mut self, filter: &'a str) -> Self {
        self.filter = Some(filter);
        self
    }

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

    /// Sends the request and returns all records.
    ///
    /// This method automatically handles pagination by making multiple requests
    /// if necessary to fetch all records matching the query.
    ///
    /// # Errors
    ///
    /// Returns a `RequestError` if:
    /// - The API request fails
    /// - Authentication is required but not provided
    /// - The response cannot be parsed
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
