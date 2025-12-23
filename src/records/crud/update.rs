use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::{BadRequestError, BadRequestResponse};
use crate::{Collection, PocketBase};

/// Represents the various errors that can be obtained after a `update` request.
#[derive(Error, Debug)]
pub enum UpdateError {
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
    #[error(
        "Could not parse response into the expected data structure. It usually means that there is a missmatch between the provided Generic Type Parameter and your Collection definition: {0}"
    )]
    ParseError(String),
    /// The response from the `PocketBase` instance API was unexpected.
    /// If you think its an error, please [open an issue on GitHub]("https://github.com/fromhorizons/pocketbase-rs/issues").
    #[error("An unhandled status code was returned by the PocketBase API: {0}")]
    UnexpectedResponse(String),
}

pub struct CollectionUpdateBuilder<'a, T: Send + Serialize + Deserialize<'a>> {
    client: &'a PocketBase,
    collection_name: &'a str,
    record_id: &'a str,
    data: T,
    _marker: std::marker::PhantomData<T>,
}

// TODO: Include the actual record data based on Generic type parameter.
// 
// pub struct UpdateResponse<T> {
//     pub collection_name: String,
//     pub collection_id: String,
//     pub id: String,
//     pub updated: String,
//     pub created: String,
//     #[serde(flatten)]
//     pub record: T, // The actual record data
// }

/// Contains information about the successfully updated Record
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResponse {
    pub collection_name: String,
    pub collection_id: String,
    pub id: String,
    pub updated: String,
    pub created: String,
}

impl<'a> Collection<'a> {
    /// Update a single record.
    ///
    /// # Example
    /// ```rust,ignore
    /// #[derive(Default, Serialize, Clone, Debug)]
    /// struct Article {
    ///     name: String,
    ///     content: String,
    /// }
    ///
    /// let updated_article = Article {
    ///     name: String::from("Updated Article Title"),
    ///     content: String::from("Updated article content"),
    /// };
    ///
    /// let response = pb
    ///     .collection("articles")
    ///     .update::<Article>("record_id_123", updated_article)
    ///     .await?;
    /// ```
    pub async fn update<T: Default + Serialize + Clone + Send>(
        self,
        record_id: &'a str,
        record: T,
    ) -> Result<UpdateResponse, UpdateError> {
        let collection_name = self.name;

        let endpoint = format!(
            "{}/api/collections/{}/records/{}",
            self.client.base_url, collection_name, record_id
        );

        let request = self
            .client
            .request_patch_json(&endpoint, &record)
            .send()
            .await;

        match request {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => {
                    let data = response.json::<UpdateResponse>().await;

                    match data {
                        Ok(data) => Ok(data),
                        Err(error) => Err(UpdateError::ParseError(error.to_string())),
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

                            Err(UpdateError::BadRequest(errors))
                        }
                        Err(error) => Err(UpdateError::ParseError(error.to_string())),
                    }
                }

                reqwest::StatusCode::FORBIDDEN => Err(UpdateError::Forbidden),
                reqwest::StatusCode::NOT_FOUND => Err(UpdateError::NotFound),

                _ => Err(UpdateError::UnexpectedResponse(
                    response.status().to_string(),
                )),
            },

            Err(error) => Err(UpdateError::Unreachable(error.to_string())),
        }
    }
}
