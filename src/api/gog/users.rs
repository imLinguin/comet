use crate::api::structs::Token;
use reqwest::{Client, Error};
use tokio::time;

pub async fn get_token_for(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
    session: &Client,
) -> Result<Token, Error> {
    let mut url = reqwest::Url::parse(
        "https://auth.gog.com/token?grant_type=refresh_token&without_new_session=1",
    )
    .unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("client_secret", client_secret)
        .append_pair("refresh_token", refresh_token);

    let result = session
        .get(url)
        .timeout(time::Duration::from_secs(10))
        .send()
        .await?;

    let result = result.error_for_status()?;
    let token: Token = result.json().await?;
    Ok(token)
}
