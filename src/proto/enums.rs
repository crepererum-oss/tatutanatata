use anyhow::Result;
use serde::{de::Error, Deserializer, Serializer};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ArchiveDataType {
    AuthorityRequests,
    Attachments,
    MailDetails,
}

impl serde::Serialize for ArchiveDataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::AuthorityRequests => "0",
            Self::Attachments => "1",
            Self::MailDetails => "2",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> serde::Deserialize<'de> for ArchiveDataType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "0" => Ok(Self::AuthorityRequests),
            "1" => Ok(Self::Attachments),
            "2" => Ok(Self::MailDetails),
            s => Err(D::Error::custom(format!("invalid archive data type: {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::testing::{assert_deser_error, assert_roundtrip};

    use super::*;

    #[test]
    fn test_roundtrip_kdf_version() {
        assert_roundtrip(KdfVersion::Bcrypt, r#""0""#);
        assert_roundtrip(KdfVersion::Argon2id, r#""1""#);

        assert_deser_error::<KdfVersion>(r#""2""#, "invalid KDF version: 2");
    }

    #[test]
    fn test_roundtrip_group_type() {
        assert_roundtrip(GroupType::User, r#""0""#);
        assert_roundtrip(GroupType::Admin, r#""1""#);
        assert_roundtrip(GroupType::MailingList, r#""2""#);
        assert_roundtrip(GroupType::Customer, r#""3""#);
        assert_roundtrip(GroupType::External, r#""4""#);
        assert_roundtrip(GroupType::Mail, r#""5""#);
        assert_roundtrip(GroupType::Contact, r#""6""#);
        assert_roundtrip(GroupType::File, r#""7""#);
        assert_roundtrip(GroupType::LocalAdmin, r#""8""#);
        assert_roundtrip(GroupType::Calendar, r#""9""#);
        assert_roundtrip(GroupType::Template, r#""10""#);
        assert_roundtrip(GroupType::ContactList, r#""11""#);

        assert_deser_error::<GroupType>(r#""20""#, "invalid group type: 20");
    }

    #[test]
    fn test_roundtrip_mail_folder_type() {
        assert_roundtrip(MailFolderType::Custom, r#""0""#);
        assert_roundtrip(MailFolderType::Inbox, r#""1""#);
        assert_roundtrip(MailFolderType::Sent, r#""2""#);
        assert_roundtrip(MailFolderType::Trash, r#""3""#);
        assert_roundtrip(MailFolderType::Archive, r#""4""#);
        assert_roundtrip(MailFolderType::Spam, r#""5""#);
        assert_roundtrip(MailFolderType::Draft, r#""6""#);

        assert_deser_error::<MailFolderType>(r#""20""#, "invalid mail folder type: 20");
    }

    #[test]
    fn test_roundtrip_archive_data_type() {
        assert_roundtrip(ArchiveDataType::AuthorityRequests, r#""0""#);
        assert_roundtrip(ArchiveDataType::Attachments, r#""1""#);
        assert_roundtrip(ArchiveDataType::MailDetails, r#""2""#);

        assert_deser_error::<ArchiveDataType>(r#""20""#, "invalid archive data type: 20");
    }
}
