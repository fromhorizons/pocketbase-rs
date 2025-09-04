use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::{BadRequestError, BadRequestResponse};
use crate::Collection;

/// Represents the various errors that can be obtained after a `create` request.
#[derive(Error, Debug)]
pub enum CreateError {
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [400 Bad Request]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400") HTTP error response.
    ///
    /// One or more fields were not validated `PocketBase`.
    #[error("One or more fields were not validated : {0:?}")]
    BadRequest(Vec<BadRequestError>),
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [403 Forbidden]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403") HTTP error response.
    ///
    /// The authorized record is not allowed to perform this action.
    #[error("The authorized record is not allowed to perform this action.")]
    Forbidden,
    /// Communication with the `PocketBase` API was successful,
    /// but returned a [404 Not Found]("https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404") HTTP error response.
    ///
    /// The requested resource wasn't found. Missing collection context.
    #[error("The requested resource wasn't found. Missing collection context.")]
    NotFound,
    /// Communication with the `PocketBase` API failed.
    ///
    /// This could be caused by an internet outage, an error in the link given to the `PocketBase` SDK
    /// and similar errors.
    #[error("The communication with the PocketBase API failed: {0}")]
    Unreachable(String),
    /// The response could not be parsed into the expected data structure.
    #[error("Could not parse response into the expected data structure. It usually means that there is a mismatch between the provided Generic Type Parameter and your Collection definition: {0}")]
    ParseError(String),
    /// An unexpected error occurred.
    /// The response from the `PocketBase` instance API was unexpected.
    /// If you think its an error, please [open an issue on GitHub]("https://github.com/fromhorizons/pocketbase-rs/issues").
    #[error("An unhandled status code was returned by the PocketBase API: {0}")]
    UnexpectedResponse(String),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateResponse {
    pub collection_name: String,
    pub collection_id: String,
    pub id: String,
    pub updated: String,
    pub created: String,
}

impl Collection<'_> {
    /// Create a new record in the given collection, from the given struct.
    ///
    /// If you need to upload files, you may want [`Collection::create_multipart()`].
    ///
    /// The `record` parameter must implement the `Serialize` trait.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::error::Error;
    ///
    /// use pocketbase_rs::PocketBase;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Default, Serialize, Deserialize, Clone, Debug)]
    /// pub struct Article {
    ///     name: String,
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
    ///         .create::<Article>(Article {
    ///             name: "test".to_string(),
    ///             content: "an interesting article content.".to_string(),
    ///         })
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The request to the server fails (`CreateError::Unreachable`).
    /// - The server responds with a bad request status (`CreateError::BadRequest`).
    /// - The server responds with a forbidden status (`CreateError::Forbidden`).
    /// - The record is not found (`CreateError::NotFound`).
    /// - The server responds with an unexpected status (`CreateError::UnexpectedResponse`).
    /// - The response could not be parsed into the expected data structure (`CreateError::ParseError`).
    pub async fn create<T: Default + Serialize + Clone + Send>(
        self,
        record: T,
    ) -> Result<CreateResponse, CreateError> {
        let endpoint = format!(
            "{}/api/collections/{}/records",
            self.client.base_url, self.name
        );

        let request = self
            .client
            .request_post_json(&endpoint, &record)
            .send()
            .await;

        create_processing(request).await
    }

    /// Create a new record in the given collection, from the given [`crate::Form`].
    ///
    /// If you don't need to upload files, you probably want the "simpler" [`Collection::create()`] method.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::{error::Error, fs};
    ///
    /// use pocketbase_rs::{Form, Part, PocketBaseAdminBuilder};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Default, Serialize, Deserialize, Clone, Debug)]
    /// pub struct Article {
    ///     name: String,
    ///     content: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut admin_pb = PocketBaseAdminBuilder::new("http://localhost:8081")
    ///         .auth_with_password("test@test.com", "abcdefghijkl")
    ///         .await?;
    ///
    ///     let image = fs::read("./vulpes_vulpes.jpg")?;
    ///
    ///     let image_part = Part::bytes(image)
    ///         .file_name("vulpes_vulpes")
    ///         .mime_str("image/jpeg")?;
    ///
    ///     let form = Form::new()
    ///         .text("name", "Red Fox")
    ///         .part("illustration", image_part);
    ///
    ///     let request = admin_pb
    ///         .collection("foxes")
    ///         .create_multipart(form)
    ///         .await;
    ///
    ///     match request {
    ///         Ok(record) => println!("Ok: {:?}", record),
    ///         Err(error) => eprintln!("Error: {error}"),
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The request to the server fails (`CreateError::Unreachable`).
    /// - The server responds with a bad request status (`CreateError::BadRequest`).
    /// - The server responds with a forbidden status (`CreateError::Forbidden`).
    /// - The record is not found (`CreateError::NotFound`).
    /// - The server responds with an unexpected status (`CreateError::UnexpectedResponse`).
    /// - The response could not be parsed into the expected data structure (`CreateError::ParseError`).
    pub async fn create_multipart(
        self,
        form: reqwest::multipart::Form,
    ) -> Result<CreateResponse, CreateError> {
        let collection_name = self.name;

        let endpoint = format!(
            "{}/api/collections/{}/records",
            self.client.base_url, collection_name
        );

        let request = self.client.request_post_form(&endpoint, form).send().await;

        create_processing(request).await
    }
}

async fn create_processing(
    request: Result<reqwest::Response, reqwest::Error>,
) -> Result<CreateResponse, CreateError> {
    match request {
        Ok(response) => match response.status() {
            reqwest::StatusCode::OK => {
                let data = response.json::<CreateResponse>().await;

                match data {
                    Ok(data) => Ok(data),
                    Err(error) => Err(CreateError::ParseError(error.to_string())),
                }
            }

            reqwest::StatusCode::BAD_REQUEST => {
                let data = response.json::<BadRequestResponse>().await;

                match data {
                    Ok(bad_response) => {
                        let mut errors: Vec<BadRequestError> = vec![];

                        for (error_name, error_data) in bad_response.data {
                            errors.push(BadRequestError {
                                name: error_name,
                                code: error_data.code,
                                message: error_data.message,
                            });
                        }

                        Err(CreateError::BadRequest(errors))
                    }
                    Err(error) => Err(CreateError::ParseError(error.to_string())),
                }
            }

            reqwest::StatusCode::FORBIDDEN => Err(CreateError::Forbidden),
            reqwest::StatusCode::NOT_FOUND => Err(CreateError::NotFound),

            _ => Err(CreateError::UnexpectedResponse(
                response.status().to_string(),
            )),
        },

        Err(error) => Err(CreateError::Unreachable(error.to_string())),
    }
}
