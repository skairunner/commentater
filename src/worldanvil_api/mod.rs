use crate::worldanvil_api::schema::{Article, ErrorBody, IdentityBody, IdentityResult, LimitOffsetBody, State, WorldArticlesResponse};
use anyhow::Context;
use reqwest::StatusCode;

pub mod schema;

const API_BASE: &str = "https://www.worldanvil.com/api/external/boromir";
const LIST_ARTICLES: &str = const_format::concatcp!(API_BASE, "/world/articles");
const USER_IDENTITY: &str = const_format::concatcp!(API_BASE, "/identity");

pub async fn world_list_articles(
    client: &reqwest::Client,
    world_id: &str,
) -> anyhow::Result<Vec<Article>> {
    let mut items = vec![];
    let mut offset = 0;
    let mut done = false;
    loop {
        let res = client
            .post(LIST_ARTICLES)
            .query(&[("id", world_id)])
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&LimitOffsetBody {
                limit: 50,
                offset,
            })?)
            .send()
            .await?;
        let text = &res.text().await?;
        let res: WorldArticlesResponse =
            serde_json::from_str(text).context(format!("Parsing json: {text}"))?;
        // If the request returned fewer than 50 responses, this means we are at the end
        if res.entities.len() < 50 {
            done = true;
        }
        offset += 50;
        // Filter out non-public articles
        let new_items: Vec<_> = res
            .entities
            .into_iter()
            .filter(|article| article.state == State::Public)
            .collect();
        items.extend(new_items);
        if done {
            break;
        }
    }
    Ok(items)
}

pub async fn get_user_identity(
    client: &reqwest::Client,
) -> anyhow::Result<IdentityResult> {
    let res = client.get(USER_IDENTITY)
        .send()
        .await?;
    match res.status() {
        StatusCode::OK => {
            Ok(IdentityResult::Identified(res.json::<IdentityBody>().await?))
        }
        StatusCode::UNAUTHORIZED => {
            Ok(IdentityResult::NotIdentified(res.json::<ErrorBody>().await?))
        }
        _ => {
            Err(res.error_for_status().unwrap_err().into())
        }
    }
}
