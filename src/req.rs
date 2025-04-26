use crate::err::AppError;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, HOST};
use reqwest::{Client, ClientBuilder, Url};
use std::env;

static USER_AGENT: &str = concat!(
    "commentater (commentater.skye.im, ",
    env!("CARGO_PKG_VERSION"),
    ")"
);

/// Get a new clientbuilder with user agent header set.
pub fn get_client_builder() -> ClientBuilder {
    ClientBuilder::new().user_agent(USER_AGENT)
}

/// Fetch a reqwest client that has the appropriate auth headers
pub fn get_wa_client_builder(user_key: &str) -> ClientBuilder {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-application-key",
        HeaderValue::from_str(&env::var("WORLDANVIL_APPLICATION_KEY").unwrap()).unwrap(),
    );
    headers.insert("x-auth-token", HeaderValue::from_str(user_key).unwrap());
    headers.insert(HOST, HeaderValue::from_str("www.worldanvil.com").unwrap());
    get_client_builder().default_headers(headers)
}

pub fn get_default_reqwest() -> Client {
    get_client_builder().build().unwrap()
}

pub fn check_url_valid(url: &str) -> Result<(), AppError> {
    let url = Url::parse(url)?;
    match url.domain() {
        None => Err(AppError::BadRequest("Url is missing domain".to_string())),
        Some(domain) => {
            if domain != "worldanvil.com" {
                Err(AppError::BadRequest(
                    "Non-worldanvil domains are not allowed".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }
}
