use serde::Deserialize;

pub mod auth_refresh;
pub mod auth_refresh_for_user;
pub mod auth_with_password;
pub mod impersonate;
pub mod request_verification;

/// Stores authentication details for a `PocketBase` user.
///
/// The `AuthStore` struct holds the authenticated user's record and a token
/// used for making authenticated requests to the `PocketBase` API.
#[derive(Clone, Debug, Deserialize)]
pub struct AuthStore {
    /// The authenticated user's record.
    pub record: AuthStoreRecord,
    /// The authentication token.
    pub token: String,
}

/// Represents the details of an authenticated user's record.
///
/// The `AuthStoreRecord` struct contains information about the user,
/// such as their ID, email, etc. and other metadata related to the
/// collection they belong to.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStoreRecord {
    /// The user's unique ID.
    pub id: String,
    /// The ID of the collection the user belongs to.
    pub collection_id: String,
    /// The name of the collection the user belongs to.
    pub collection_name: String,
    /// The timestamp when the record was created.
    pub created: String,
    /// The timestamp when the record was last updated.
    pub updated: String,
    /// The user's email address.
    pub email: String,
    /// Indicates whether the user's email is publicly visible.
    pub email_visibility: bool,
    /// Indicates whether the user's email has been verified.
    pub verified: bool,
}
