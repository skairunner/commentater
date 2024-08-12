use reqwest::{Client, ClientBuilder};


static USER_AGENT: &str = concat!(
    "commentater", "/", env!("CARGO_PKG_VERSION"), " Managed by skairunner on Discord");


/// Get a new clientbuilder with user agent header set.
pub fn get_client_builder() -> ClientBuilder {
    ClientBuilder::new()
        .user_agent(USER_AGENT)
}

pub fn get_default_reqwest() -> Client {
    get_client_builder().build().unwrap()
}
