use anyhow::Result;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::binary::{Base64String, Base64Url};

pub(crate) mod binary;

#[cfg(test)]
mod testing;

pub(crate) trait Entity {
    fn id(&self) -> &str;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Format<const F: u8>;

impl<const F: u8> serde::Serialize for Format<F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&F.to_string())
    }
}

impl<'de, const F: u8> serde::Deserialize<'de> for Format<F> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let f: u8 = s.parse().map_err(D::Error::custom)?;
        if f == F {
            Ok(Self)
        } else {
            Err(D::Error::custom(format!("invalid format: {f}")))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum KdfVersion {
    Bcrypt,
    Argon2id,
}

impl serde::Serialize for KdfVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::Bcrypt => "0",
            Self::Argon2id => "1",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> serde::Deserialize<'de> for KdfVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "0" => Ok(Self::Bcrypt),
            "1" => Ok(Self::Argon2id),
            s => Err(D::Error::custom(format!("invalid KDF version: {s}"))),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Null;

impl serde::Serialize for Null {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum GroupType {
    User,
    Admin,
    MailingList,
    Customer,
    External,
    Mail,
    Contact,
    File,
    LocalAdmin,
    Calendar,
    Template,
    ContactList,
}

impl serde::Serialize for GroupType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::User => "0",
            Self::Admin => "1",
            Self::MailingList => "2",
            Self::Customer => "3",
            Self::External => "4",
            Self::Mail => "5",
            Self::Contact => "6",
            Self::File => "7",
            Self::LocalAdmin => "8",
            Self::Calendar => "9",
            Self::Template => "10",
            Self::ContactList => "11",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> serde::Deserialize<'de> for GroupType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "0" => Ok(Self::User),
            "1" => Ok(Self::Admin),
            "2" => Ok(Self::MailingList),
            "3" => Ok(Self::Customer),
            "4" => Ok(Self::External),
            "5" => Ok(Self::Mail),
            "6" => Ok(Self::Contact),
            "7" => Ok(Self::File),
            "8" => Ok(Self::LocalAdmin),
            "9" => Ok(Self::Calendar),
            "10" => Ok(Self::Template),
            "11" => Ok(Self::ContactList),
            s => Err(D::Error::custom(format!("invalid group type: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MailFolderType {
    Custom,
    Inbox,
    Sent,
    Trash,
    Archive,
    Spam,
    Draft,
}

impl MailFolderType {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Custom => "Custom",
            Self::Inbox => "Inbox",
            Self::Sent => "Sent",
            Self::Trash => "Trash",
            Self::Archive => "Archive",
            Self::Spam => "Spam",
            Self::Draft => "Draft",
        }
    }
}

impl serde::Serialize for MailFolderType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::Custom => "0",
            Self::Inbox => "1",
            Self::Sent => "2",
            Self::Trash => "3",
            Self::Archive => "4",
            Self::Spam => "5",
            Self::Draft => "6",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> serde::Deserialize<'de> for MailFolderType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "0" => Ok(Self::Custom),
            "1" => Ok(Self::Inbox),
            "2" => Ok(Self::Sent),
            "3" => Ok(Self::Trash),
            "4" => Ok(Self::Archive),
            "5" => Ok(Self::Spam),
            "6" => Ok(Self::Draft),
            s => Err(D::Error::custom(format!("invalid mail folder type: {s}"))),
        }
    }
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
    pub(crate) sym_enc_g_key: Base64String,
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
    pub(crate) owner_enc_session_key: Base64String,

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
    pub(crate) owner_enc_session_key: Base64String,

    #[serde(rename = "_ownerGroup")]
    pub(crate) owner_group: String,

    #[serde(rename = "_id")]
    pub(crate) id: [String; 2],

    pub(crate) mail_details: [String; 2],
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
pub(crate) struct MailDetails {
    pub(crate) body: MailBody,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MailDetailsBlob {
    #[serde(rename = "_format")]
    pub(crate) _format: Format<0>,

    pub(crate) details: MailDetails,
}

#[cfg(test)]
mod tests {
    use super::testing::{assert_deser_error, assert_roundtrip};
    use super::*;

    #[test]
    fn test_roundtrip_format() {
        assert_roundtrip(Format::<0>);
        assert_roundtrip(Format::<1>);
        assert_roundtrip(Format::<2>);
        assert_roundtrip(Format::<255>);

        assert_deser_error::<Format<1>>(r#""0""#, "invalid format: 0");
    }

    #[test]
    fn test_roundtrip_kdf_version() {
        assert_roundtrip(KdfVersion::Bcrypt);
        assert_roundtrip(KdfVersion::Argon2id);

        assert_deser_error::<KdfVersion>(r#""2""#, "invalid KDF version: 2");
    }

    #[test]
    fn test_roundtrip_group_type() {
        assert_roundtrip(GroupType::User);
        assert_roundtrip(GroupType::Admin);
        assert_roundtrip(GroupType::MailingList);
        assert_roundtrip(GroupType::Customer);
        assert_roundtrip(GroupType::External);
        assert_roundtrip(GroupType::Mail);
        assert_roundtrip(GroupType::Contact);
        assert_roundtrip(GroupType::File);
        assert_roundtrip(GroupType::LocalAdmin);
        assert_roundtrip(GroupType::Calendar);
        assert_roundtrip(GroupType::Template);
        assert_roundtrip(GroupType::ContactList);

        assert_deser_error::<GroupType>(r#""20""#, "invalid group type: 20");
    }

    #[test]
    fn test_roundtrip_mail_folder_type() {
        assert_roundtrip(MailFolderType::Custom);
        assert_roundtrip(MailFolderType::Inbox);
        assert_roundtrip(MailFolderType::Sent);
        assert_roundtrip(MailFolderType::Trash);
        assert_roundtrip(MailFolderType::Archive);
        assert_roundtrip(MailFolderType::Spam);
        assert_roundtrip(MailFolderType::Draft);

        assert_deser_error::<MailFolderType>(r#""20""#, "invalid mail folder type: 20");
    }
}
