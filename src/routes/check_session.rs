use crate::auth::Session;

#[get("/check")]
pub async fn check_session(_session: Session) -> crate::util::Result<()> {
    Ok(())
}
