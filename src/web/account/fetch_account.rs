/// Fetch your account
/// GET /account

#[get("/")]
pub async fn fetch_account(auth: &State<Auth>, account: Account) -> Result<EmptyResponse> {
    //let account = Account
}

/// Requires authentication x-session-token or scope user.email

/// Responses:
// { id: String, email: String }
