#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_json;

pub mod config;
pub mod entities;
pub mod logic;
pub mod util;
pub mod web;

#[cfg(test)]
pub mod test;

/* Old Tests
#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn validation() {
        let auth = for_test("validation").await;

        assert!(auth.validate_password("unique password").await.is_ok());
        assert!(auth.validate_password("password").await.is_err());

        assert!(auth.validate_email("person@validemail.com").await.is_ok());
        assert!(auth.validate_email("invalid email").await.is_err());
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn create_account() {
        let auth = for_test("create_account").await;

        assert!(auth
            .create_account(
                "paulmakles@gmail.com".to_string(),
                "sussy".to_string(),
                false,
            )
            .await
            .is_ok());
    }
}*/
