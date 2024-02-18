use serde::{Deserialize, Serialize};

use crate::proto::constants::Format;

use super::{
    binary::{Base64String, Base64Url},
    constants::Null,
    date::UnixDate,
    enums::{GroupType, KdfVersion, MailFolderType},
    keys::{EncryptedKey, OptionalEncryptedKey},
};

pub(crate) trait Entity {
    fn id(&self) -> &str;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaltServiceRequest {
    #[serde(rename = "_format")]
    pub(crate) format: Format<0>,

    pub(crate) mail_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaltServiceResponse {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    pub(crate) kdf_version: KdfVersion,

    pub(crate) salt: Base64String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionServiceRequest {
    #[serde(rename = "_format")]
    pub(crate) format: Format<0>,

    pub(crate) access_key: Null,

    pub(crate) auth_token: Null,

    pub(crate) auth_verifier: Base64Url,

    pub(crate) client_identifier: String,

    pub(crate) mail_address: String,

    pub(crate) recover_code_verifier: Null,

    pub(crate) user: Null,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionServiceResponse {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    pub(crate) access_token: Base64Url,

    pub(crate) challenges: Vec<String>,

    pub(crate) user: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserMembership {
    pub(crate) group_type: GroupType,
    pub(crate) group: String,
    pub(crate) sym_enc_g_key: OptionalEncryptedKey,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserAuth {
    pub(crate) sessions: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserResponse {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    pub(crate) memberships: Vec<UserMembership>,
    pub(crate) auth: UserAuth,
    pub(crate) user_group: UserMembership,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MailboxGroupRootResponse {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    pub(crate) mailbox: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Folders {
    pub(crate) folders: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MailboxResponse {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    pub(crate) folders: Folders,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FolderResponse {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    #[serde(rename = "_id")]
    pub(crate) id: [String; 2],

    #[serde(rename = "_ownerEncSessionKey")]
    pub(crate) owner_enc_session_key: EncryptedKey,

    #[serde(rename = "_ownerGroup")]
    pub(crate) owner_group: String,

    pub(crate) folder_type: MailFolderType,
    pub(crate) name: Base64String,
    pub(crate) mails: String,
}

impl Entity for FolderResponse {
    fn id(&self) -> &str {
        &self.id[1]
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MailReponse {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    #[serde(rename = "_ownerEncSessionKey")]
    pub(crate) owner_enc_session_key: EncryptedKey,

    #[serde(rename = "_ownerGroup")]
    pub(crate) owner_group: String,

    #[serde(rename = "_id")]
    pub(crate) id: [String; 2],

    pub(crate) mail_details: [String; 2],

    pub(crate) received_date: UnixDate,
    pub(crate) subject: Base64String,
}

impl Entity for MailReponse {
    fn id(&self) -> &str {
        &self.id[1]
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlobReadRequest {
    #[serde(rename = "_id")]
    pub(crate) id: String,

    pub(crate) archive_id: String,
    pub(crate) instance_ids: Vec<()>,
    pub(crate) instance_list_id: Null,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlobAccessTokenServiceRequest {
    #[serde(rename = "_format")]
    pub(crate) format: Format<0>,

    pub(crate) archive_data_type: Null,
    pub(crate) read: BlobReadRequest,
    pub(crate) write: Null,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlobServer {
    pub(crate) url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlobAccessInfo {
    pub(crate) blob_access_token: String,
    pub(crate) servers: Vec<BlobServer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlobAccessTokenServiceResponse {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    pub(crate) blob_access_info: BlobAccessInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MailBody {
    pub(crate) compressed_text: Base64String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MailHeaders {
    pub(crate) compressed_headers: Base64String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MailDetails {
    pub(crate) body: MailBody,

    /// Mail headers.
    ///
    /// These only appear for true emails, not for internal messages.
    pub(crate) headers: Option<MailHeaders>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MailDetailsBlob {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    pub(crate) details: MailDetails,
}
