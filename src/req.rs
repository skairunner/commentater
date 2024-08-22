use crate::err::AppError;
use reqwest::{Client, ClientBuilder, Url};

static USER_AGENT: &str = concat!(
    "commentater",
    "/",
    env!("CARGO_PKG_VERSION"),
    " Managed by skairunner on Discord"
);

/// Get a new clientbuilder with user agent header set.
pub fn get_client_builder() -> ClientBuilder {
    ClientBuilder::new().user_agent(USER_AGENT)
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
