// perPage default: 30
// perPage max:     500
// filter, sort, expand, page, perPage, skipTotal

use serde::{de::DeserializeOwned, Deserialize};

use crate::error::RequestError;
use crate::PocketBase;
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
    /// Fetch a paginated records list from the given collection, supporting sorting and filtering.
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
    ///     id: String,
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
    ///     let articles = pb
    ///         .collection("articles")
    ///         .get_list::<Article>()
    ///         .sort("-created,id")
    ///         .call()
    ///         .await?;
    ///
    ///     for article in articles.items {
    ///         println!("{article:?}");
    ///     }
    ///
    ///     Ok(())
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

    /// Specify the max returned records per page (default to 30).
    ///
    /// If a value greater than **500** is provided, `PocketBase` will
    /// automatically limit it to **500**.
    pub fn per_page(mut self, per_page: u16) -> Self {
        self.per_page = Some(per_page.to_string());
        self
    }

    /// Specify the records order attribute(s).
    /// Add `-`/`+` (default) in front of the attribute for DESC / ASC order.
    ///
    /// Example:
    /// ```toml
    /// ?sort=-created,id # DESC by created and ASC by id
    /// ``````
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
    ///    - `=`     Equal
    ///    - `!=`   NOT equal
    ///    - `>`     Greater than
    ///    - `>=`   Greater than or equal
    ///    - `<`     Less than
    ///    - `<=`   Less than or equal
    ///    - `~`     Like/Contains (if not specified auto wraps the right string OPERAND in a "%" for wildcard match)
    ///    - `!~`   NOT Like/Contains (if not specified auto wraps the right string OPERAND in a "%" for wildcard match)
    ///    - `?=`    *Any/At least one of* Equal
    ///    - `?!=`  *Any/At least one of* NOT equal
    ///    - `?>`    *Any/At least one of* Greater than
    ///    - `?>=`  *Any/At least one of* Greater than or equal
    ///    - `?<`    *Any/At least one of* Less than
    ///    - `?<=`  *Any/At least one of* Less than or equal
    ///    - `?~`    *Any/At least one of* Like/Contains (if not specified auto wraps the right string OPERAND in a "%" for wildcard match)
    ///    - `?!~` *Any/At least one of* NOT Like/Contains (if not specified auto wraps the right string OPERAND in a "%" for wildcard match)
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

    /// If it is set the total counts query will be skipped and the response fields `totalItems` and `totalPages` will have `-1` value.
    /// This could drastically speed up the search queries when the total counters are not needed or cursor speed pagination is used.
    /// For optimization purposes, it is set by default for the `getFirstListItem()` and `getFullList()` SDKs methods.
    pub const fn skip_total(mut self, skip_total: bool) -> Self {
        self.skip_total = skip_total;
        self
    }

    /// Sends the request and returns the response.
    ///
    /// This method finalizes the request built using the builder pattern
    /// and sends it to the API endpoint. It should be called after all
    /// desired parameters and configurations have been set on the builder.
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
