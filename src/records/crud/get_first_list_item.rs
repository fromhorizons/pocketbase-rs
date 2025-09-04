// set SkipTotal to true for performances

use serde::{de::DeserializeOwned, Deserialize};

use crate::error::RequestError;
use crate::PocketBase;
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
    /// Fetch first record from the given collection, supporting sorting and filtering.
    ///
    /// This function returns a `CollectionGetFirstListItemBuilder`, which allows you to specify
    /// additional options such as sorting, filtering or expanding linked records before calling
    /// `.call().await` to execute the request.
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
    ///     language: String
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
    ///         .get_first_list_item::<Article>()
    ///         .filter("language='en'")
    ///         .call()
    ///         .await?;
    ///
    ///     println!("Article: {article:?}");
    ///
    ///     Ok(())
    /// }
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

    /// Sends the request and returns the response.
    ///
    /// This method finalizes the request built using the builder pattern
    /// and sends it to the API endpoint. It should be called after all
    /// desired parameters and configurations have been set on the builder.
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
