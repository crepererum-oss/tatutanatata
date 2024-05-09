use anyhow::Result;
use serde::{de::Error, Deserializer, Serializer};

macro_rules! build_enum {
    ($name:ident, [$($element:ident = $descr:expr,)*] $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) enum $name {
            $(
                $element,
            )*
        }

        impl $name {
            #[allow(dead_code)]
            pub(crate) fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$element => stringify!($element),
                    )*
                }
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let s = match self {
                    $(
                        Self::$element => $descr,
                    )*
                };

                serializer.serialize_str(s)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;

                match s.as_str() {
                    $(
                        $descr => Ok(Self::$element),
                    )*
                    s => Err(D::Error::custom(format!("unknown variant: {s}"))),
                }
            }
        }
    };
}

build_enum!(KdfVersion, [Bcrypt = "0", Argon2id = "1",]);

build_enum!(
    GroupType,
    [
        User = "0",
        Admin = "1",
        MailingList = "2",
        Customer = "3",
        External = "4",
        Mail = "5",
        Contact = "6",
        File = "7",
        LocalAdmin = "8",
        Calendar = "9",
        Template = "10",
        ContactList = "11",
    ],
);

build_enum!(
    MailFolderType,
    [
        Custom = "0",
        Inbox = "1",
        Sent = "2",
        Trash = "3",
        Archive = "4",
        Spam = "5",
        Draft = "6",
    ],
);

build_enum!(
    ArchiveDataType,
    [
        AuthorityRequests = "0",
        Attachments = "1",
        MailDetails = "2",
    ],
);

#[cfg(test)]
mod tests {
    use crate::proto::testing::{assert_deser_error, assert_roundtrip};

    use super::*;

    #[test]
    fn test_roundtrip_kdf_version() {
        assert_roundtrip(KdfVersion::Bcrypt, r#""0""#);
        assert_roundtrip(KdfVersion::Argon2id, r#""1""#);

        assert_deser_error::<KdfVersion>(r#""2""#, "unknown variant: 2");
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

        assert_deser_error::<GroupType>(r#""20""#, "unknown variant: 20");
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

        assert_deser_error::<MailFolderType>(r#""20""#, "unknown variant: 20");
    }

    #[test]
    fn test_roundtrip_archive_data_type() {
        assert_roundtrip(ArchiveDataType::AuthorityRequests, r#""0""#);
        assert_roundtrip(ArchiveDataType::Attachments, r#""1""#);
        assert_roundtrip(ArchiveDataType::MailDetails, r#""2""#);

        assert_deser_error::<ArchiveDataType>(r#""20""#, "unknown variant: 20");
    }
}
