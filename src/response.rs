#[derive(serde::Serialize)]
pub struct RegisterArticleResponse {
    pub id: i64,
}

#[derive(serde::Serialize)]
pub struct AppJsonError {
    pub error: String,
}
