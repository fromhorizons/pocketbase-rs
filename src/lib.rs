//! `pocketbase-rs` is a Rust wrapper around `PocketBase`'s REST API.
//!
//! # Usage
//!
//! ```rust,ignore
//! use std::error::Error;
//!
//! use pocketbase_rs::{PocketBase, Collection, RequestError};
//! use serde::Deserialize;
//!
//! #[derive(Default, Deserialize, Clone)]
//! struct Article {
//!     title: String,
//!     content: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let mut pb = PocketBase::new("http://localhost:8090");
//!
//!     let auth_data = pb
//!         .collection("users")
//!         .auth_with_password("YOUR_EMAIL_OR_USERNAME", "YOUR_PASSWORD")
//!         .await?;
//!
//!     let article: Article = pb
//!         .collection("articles")
//!         .get_one::<Article>("record_id_123")
//!         .call()
//!         .await?;
//!
//!     println!("Article Title: {}", article.title);
//!
//!     Ok(())
//! }
//! ```

#![deny(missing_docs)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code)]

pub use error::*;
pub use records::auth::{AuthStore, AuthStoreRecord};
use reqwest::RequestBuilder;
pub use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

pub mod error;
pub(crate) mod records;

/// Represents a specific collection in a `PocketBase` database.
///
/// The `Collection` struct provides an interface for interacting with a specific collection
/// within a `PocketBase` instance. Instances of this struct are created using the
/// [`PocketBase::collection`] method. All operations on the target collection, such as retrieving,
/// creating, updating, or deleting records, are accessible through methods implemented on
/// this struct.
///
/// # Fields
/// - `client`: A mutable reference to the `PocketBase` client instance.
///   This allows the `Collection` to send requests to `PocketBase`.
/// - `name`: The name of the collection being interacted with.
pub struct Collection<'a> {
    pub(crate) client: &'a mut PocketBase,
    pub(crate) name: &'a str,
}

impl PocketBase {
    /// Creates a new [`Collection`] instance for the specified collection name.
    ///
    /// This method provides access to operations related to a specific collection in `PocketBase`.
    /// Most interactions with the `PocketBase` API are performed through the [`Collection`] instance returned
    /// by this method.
    ///
    /// # Arguments
    /// * `collection_name` - The name of the collection to interact with, provided as a static string.
    ///
    /// # Returns
    /// A [`Collection`] instance configured for the specified collection.
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut pb = PocketBase::new("http://localhost:8090");
    ///
    /// pb.collection("users")
    ///     .auth_with_password("YOUR_EMAIL_OR_USERNAME", "YOUR_PASSWORD")
    ///     .await?;
    ///
    /// let article = pb
    ///     .collection("articles")
    ///     .get_first_list_item::<Article>()
    ///     .filter("language='en'")
    ///     .call()
    ///     .await?;
    /// ```
    ///
    /// # Panics
    ///
    /// This method will panic if the collection name is empty or contains invalid characters.
    pub fn collection(&mut self, collection_name: &'static str) -> Collection {
        // Validate collection name
        assert!(
            !collection_name.is_empty(),
            "Collection name cannot be empty"
        );

        // Collection names should only contain alphanumeric characters and underscores
        assert!(
            collection_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_'),
            "Collection name contains invalid characters. Only alphanumeric characters and underscores are allowed"
        );

        Collection {
            client: self,
            name: collection_name,
        }
    }
}

/// Represents a paginated list of records retrieved from a `PocketBase` collection.
///
/// The `RecordList` struct encapsulates the results of a paginated query to a collection.
/// It contains metadata about the pagination state (such as the current page, total items,
/// and total pages) as well as the records themselves.
///
/// This struct is typically returned by methods that fetch a list of records from a
/// collection, such as [`Collection::get_list`].
///
/// # Type Parameters
/// - `T`: The type of the records contained in the `items` list. This is typically a
///   deserialized struct that matches the schema of the records in the collection.
///
/// # Fields
/// - `page`: The current page number (starting from 1).
/// - `per_page`: The maximum number of records returned per page (default is 30).
/// - `total_items`: The total number of records in the collection that match the query.
/// - `total_pages`: The total number of pages available for the query.
/// - `items`: A vector containing the records for the current page.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordList<T> {
    /// The page (aka. offset) of the paginated list *(default to 1)*.
    pub page: i32,
    /// The max returned records per page *(default to 30)*.
    pub per_page: i32,
    /// The total amount of records found in the collection.
    pub total_items: i32,
    /// The total amount of pages found in the collection.
    pub total_pages: i32,
    /// A list of all records for the given page.
    pub items: Vec<T>,
}

/// Response structure for API errors from `PocketBase`.
#[derive(Deserialize, Debug)]
pub(crate) struct ErrorResponse {
    /// HTTP status code
    pub code: u16,
    /// Error message from the server
    pub message: String,
    /// Additional error data, if any
    pub data: Option<serde_json::Value>,
}

/// A `PocketBase` client for sending requests to a `PocketBase` instance.
///
/// The `Debug` implementation for this struct redacts sensitive authentication data
/// to prevent accidental exposure in logs.
///
/// # Example
/// ```rust,ignore
/// use std::error::Error;
/// use pocketbase_rs::PocketBase;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Article {
///     id: String,
///     title: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn Error>> {
///     let mut pb = PocketBase::new("http://localhost:8090");
///
///     pb.collection("users")
///         .auth_with_password("YOUR_EMAIL_OR_USERNAME", "YOUR_PASSWORD")
///         .await?;
///
///     let article = pb
///         .collection("articles")
///         .get_one::<Article>("record_id")
///         .call()
///         .await?;
///
///     println!("Article: {:?}", article);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct PocketBase {
    pub(crate) base_url: String,
    pub(crate) auth_store: Option<AuthStore>,
    pub(crate) reqwest_client: reqwest::Client,
}

impl std::fmt::Debug for PocketBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PocketBase")
            .field("base_url", &self.base_url)
            .field(
                "auth_store",
                &self.auth_store.as_ref().map(|_| "***REDACTED***"),
            )
            .field("reqwest_client", &"Client")
            .finish()
    }
}

impl PocketBase {
    /// Creates a new instance of the `PocketBase` client.
    ///
    /// # Example
    /// ```rust
    /// let pb = PocketBase::new("http://localhost:8090");
    /// // Use the client for further operations like authentication or fetching records
    /// ```
    /// # Panics
    ///
    /// This method will panic if the provided `base_url` is not a valid URL.
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        // Validate URL format
        let trimmed_url = base_url.trim_end_matches('/');
        assert!(
            trimmed_url.starts_with("http://") || trimmed_url.starts_with("https://"),
            "Invalid base_url: must start with http:// or https://"
        );

        // Create client with sensible defaults
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url: trimmed_url.to_string(),
            auth_store: None,
            reqwest_client: client,
        }
    }

    /// Creates a new `PocketBase` client with a custom reqwest client.
    ///
    /// # Example
    /// ```rust
    /// use std::time::Duration;
    ///
    /// let reqwest_client = reqwest::Client::builder()
    ///     .timeout(Duration::from_secs(60))
    ///     .build()
    ///     .expect("Failed to build client");
    ///
    /// let pb = PocketBase::new_with_client("http://localhost:8090", reqwest_client);
    /// ```
    ///
    /// # Panics
    ///
    /// This method will panic if the provided `base_url` is not a valid URL.
    #[must_use]
    pub fn new_with_client(base_url: &str, client: reqwest::Client) -> Self {
        // Validate URL format
        let trimmed_url = base_url.trim_end_matches('/');
        assert!(
            trimmed_url.starts_with("http://") || trimmed_url.starts_with("https://"),
            "Invalid base_url: must start with http:// or https://"
        );

        Self {
            base_url: trimmed_url.to_string(),
            auth_store: None,
            reqwest_client: client,
        }
    }

    /// Retrieves the current auth store, if available.
    ///
    /// # Example
    /// ```rust,ignore
    /// let pb = PocketBase::new("http://localhost:8090");
    ///
    /// // ...
    ///
    /// if let Some(auth_store) = pb.auth_store() {
    ///     println!("Authenticated with token: {}", auth_store.token);
    /// } else {
    ///     println!("Not authenticated");
    /// }
    /// ```
    #[must_use]
    pub fn auth_store(&self) -> Option<AuthStore> {
        self.auth_store.clone()
    }

    /// Retrieves the current authentication token, if available.
    ///
    /// # Example
    /// ```rust,ignore
    /// let pb = PocketBase::new("http://localhost:8090");
    ///
    /// // ...
    ///
    /// if let Some(token) = pb.token() {
    ///     println!("Authenticated with token: {}", token);
    /// } else {
    ///     println!("Not authenticated");
    /// }
    /// ```
    #[must_use]
    pub fn token(&self) -> Option<String> {
        self.auth_store
            .as_ref()
            .map(|auth_store| auth_store.token.clone())
    }

    /// Returns the base URL of the `PocketBase` server.
    ///
    /// # Example
    /// ```rust,ignore
    /// let pb = PocketBase::new("http://localhost:8090");
    /// assert_eq!(pb.base_url(), "http://localhost:8090".to_string());
    /// ```
    #[must_use]
    pub fn base_url(&self) -> String {
        self.base_url.clone()
    }

    pub(crate) fn update_auth_store(&mut self, new_auth_store: AuthStore) {
        self.auth_store = Some(new_auth_store);
    }
}

impl PocketBase {
    /// Adds an authorization token to the request, if available.
    ///
    /// This method attaches a bearer authentication token to the provided `RequestBuilder`
    /// if the client is currently authenticated. If no token is available, the request is
    /// returned unchanged.
    ///
    /// # Arguments
    /// * `request_builder` - A `reqwest::RequestBuilder` to which the token will be added.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` with the authorization token, if applicable.
    pub(crate) fn with_authorization_token(
        &self,
        request_builder: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        if let Some(auth_store) = self.auth_store() {
            request_builder.bearer_auth(auth_store.token)
        } else {
            request_builder
        }
    }

    /// Creates a POST request builder for the specified endpoint.
    ///
    /// This method initializes a `POST` request to the given endpoint and adds
    /// an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `POST` request to.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `POST` request.
    pub(crate) fn request_post(&self, endpoint: &str) -> RequestBuilder {
        let request_builder = self.reqwest_client.post(endpoint);
        self.with_authorization_token(request_builder)
    }

    /// Creates a PATCH request builder with JSON body for the specified endpoint.
    ///
    /// This method initializes a `PATCH` request to the given endpoint with a JSON body,
    /// and adds an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `PATCH` request to.
    /// * `params` - A reference to a serializable type to use as the JSON body of the request.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `PATCH` request.
    pub(crate) fn request_patch_json<T: Default + Serialize + Clone + Send>(
        &self,
        endpoint: &str,
        params: &T,
    ) -> RequestBuilder {
        let request_builder = self.reqwest_client.patch(endpoint).json(&params);
        self.with_authorization_token(request_builder)
    }

    /// Creates a POST request builder with JSON body for the specified endpoint.
    ///
    /// This method initializes a `POST` request to the given endpoint with a JSON body,
    /// and adds an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `POST` request to.
    /// * `params` - A reference to a serializable type to use as the JSON body of the request.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `POST` request.
    pub(crate) fn request_post_json<T: Default + Serialize + Clone + Send>(
        &self,
        endpoint: &str,
        params: &T,
    ) -> RequestBuilder {
        let request_builder = self.reqwest_client.post(endpoint).json(&params);
        self.with_authorization_token(request_builder)
    }

    /// Creates a POST request builder with a form body for the specified endpoint.
    ///
    /// This method initializes a `POST` request to the given endpoint with a multipart form body,
    /// and adds an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `POST` request to.
    /// * `form` - A `reqwest::multipart::Form` representing the form data for the request.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `POST` request.
    pub(crate) fn request_post_form(&self, endpoint: &str, form: Form) -> RequestBuilder {
        let request_builder = self.reqwest_client.post(endpoint).multipart(form);
        self.with_authorization_token(request_builder)
    }

    /// Creates a GET request builder for the specified endpoint.
    ///
    /// This method initializes a `GET` request to the given endpoint, adds an `Accept` header
    /// for JSON responses, attaches query parameters if provided, and adds an authorization
    /// token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `GET` request to.
    /// * `params` - An optional vector of key-value pairs to include as query parameters.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `GET` request.
    pub(crate) fn request_get(
        &self,
        endpoint: &str,
        params: Option<Vec<(&str, &str)>>,
    ) -> RequestBuilder {
        let mut request_builder = self
            .reqwest_client
            .get(endpoint)
            .header("Accept", "application/json");

        if let Some(params) = params {
            request_builder = request_builder.query(&params);
        }

        self.with_authorization_token(request_builder)
    }

    /// Creates a DELETE request builder for the specified endpoint.
    ///
    /// This method initializes a `DELETE` request to the given endpoint and adds
    /// an authorization token if available.
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to send the `DELETE` request to.
    ///
    /// # Returns
    /// A `reqwest::RequestBuilder` for the `DELETE` request.
    ///
    /// # Example
    /// ```rust,ignore
    /// let pb = PocketBase::new("http://localhost:8090");
    ///
    /// let request = pb.request_delete("http://localhost:8090/api/collections/articles/record_id");
    /// ```
    pub(crate) fn request_delete(&self, endpoint: &str) -> RequestBuilder {
        let request_builder = self.reqwest_client.delete(endpoint);

        self.with_authorization_token(request_builder)
    }
}
