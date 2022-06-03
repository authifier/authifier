use std::collections::HashSet;

use crate::{Error, Result};

#[derive(Serialize, Deserialize, Clone)]
pub enum EmailBlockList {
    /// Don't block any emails
    Disabled,
    /// Block a custom list of domains
    Custom { domains: HashSet<String> },
    /// Disposable mail list maintained by revolt.chat
    #[cfg(feature = "revolt_source_list")]
    RevoltSourceList,
}

#[cfg(feature = "revolt_source_list")]
impl Default for EmailBlockList {
    fn default() -> EmailBlockList {
        EmailBlockList::RevoltSourceList
    }
}

#[cfg(not(feature = "revolt_source_list"))]
impl Default for EmailBlockList {
    fn default() -> EmailBlockList {
        EmailBlockList::Disabled
    }
}

#[cfg(feature = "revolt_source_list")]
lazy_static! {
    /// Default email block list
    static ref REVOLT_SOURCE_LIST: HashSet<String> = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/revolt_source_list.txt"))
        .split('\n')
        .map(|x| x.into())
        .collect();
}

impl EmailBlockList {
    /// Get active list
    pub fn get_list(&self) -> Option<&HashSet<String>> {
        match self {
            EmailBlockList::Disabled => None,
            EmailBlockList::Custom { domains } => Some(domains),
            #[cfg(feature = "revolt_source_list")]
            EmailBlockList::RevoltSourceList => Some(&*REVOLT_SOURCE_LIST),
        }
    }

    /// Validate a given email is allowed
    pub fn validate_email(&self, email: &str) -> Result<()> {
        // Make sure this is an actual email
        if !validator::validate_email(email) {
            return Err(Error::IncorrectData { with: "email" });
        }

        // Check if the email is blacklisted
        if let Some(list) = self.get_list() {
            if let Some(domain) = email.split('@').last() {
                if list.contains(&domain.to_string()) {
                    return Err(Error::Blacklisted);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::config::EmailBlockList;
    use crate::Error;

    #[test]
    fn it_accepts_valid_emails() {
        let list = EmailBlockList::Disabled;
        assert_eq!(list.validate_email("valid@example.com"), Ok(()));
    }

    #[test]
    fn it_rejects_invalid_emails() {
        let list = EmailBlockList::Disabled;
        assert_eq!(
            list.validate_email("invalid"),
            Err(Error::IncorrectData { with: "email" })
        );
    }

    #[test]
    fn it_rejects_blocked_emails() {
        let list = EmailBlockList::Custom {
            domains: HashSet::from(["example.com".to_string()]),
        };

        assert_eq!(
            list.validate_email("test@example.com"),
            Err(Error::Blacklisted)
        );
    }
}
