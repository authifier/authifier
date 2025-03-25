//! Redirect to authorization interface
//! GET /sso/authorize
use authifier::{Authifier, Error, Result};
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::time::Duration;
use rocket::State;

/// # Redirect to authorization interface
///
/// Redirect to authorization interface.
#[openapi(tag = "SSO")]
#[get("/sso/authorize/<idp_id>?<redirect_uri>")]
pub async fn authorize(
    authifier: &State<Authifier>,
    idp_id: &str,
    redirect_uri: &str,
    cookies: &CookieJar<'_>,
) -> Result<Redirect> {
    // Make sure the redirect URI is valid
    let Ok(redirect_uri) = redirect_uri.parse() else {
        return Err(Error::InvalidRedirectUri);
    };

    // Ensure given ID provider exists
    let id_provider = authifier
        .config
        .sso
        .get(idp_id)
        .ok_or(Error::InvalidIdpId)?;

    // Build authorization URI
    let (state, uri) = id_provider
        .create_authorization_uri(authifier, &redirect_uri)
        .await?;

    // Build cookie that can be retrieved during callback
    let (path, max_age) = ("/sso/callback", Duration::seconds(60 * 10));
    let cookie = Cookie::build(("callback-id", state)).http_only(true);

    // Add the cookie to the response
    cookies.add(cookie.path(path).max_age(max_age));

    Ok(Redirect::found(uri.to_string()))
}
