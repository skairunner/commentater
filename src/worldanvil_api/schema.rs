use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum State {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum EntityClass {
    Ethnicity,
    Article,
    Landmark,
    Location,
    Ritual,
    Myth,
    Technology,
    Spell,
    Law,
    Prose,
    MilitaryConflict,
    Language,
    Document,
    Person,
    Organization,
    Plot,
    Species,
    Vehicle,
    Profession,
    Item,
    Formation,
    Rank,
    Condition,
    Material,
    Settlement,
    Report,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Date {
    pub date: String,
    pub timezone_type: i32,
    pub timezone: String,
}

/// I don't care about this so let's ignore it for now
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SubscriberGroup {}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub state: State,
    pub is_wip: bool,
    pub is_draft: bool,
    pub entity_class: EntityClass,
    pub icon: String,
    pub url: String,
    pub folder_id: String,
    pub tags: Option<String>,
    pub update_date: Date,
    pub subscribergroups: Vec<SubscriberGroup>,
    pub position: Option<i64>,
}

#[derive(Deserialize)]
pub struct WorldArticlesResponse {
    pub success: bool,
    pub entities: Vec<Article>,
}

fn serialize_i64_as_string<S: Serializer>(i: &i64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&i64::to_string(i))
}

/// A body containing "limit" and "offset", very common in the WA API.
#[derive(Serialize)]
pub struct LimitOffsetBody {
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub limit: i64,
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub offset: i64,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct IdentityBody {
    pub id: String,
    pub success: bool,
    pub username: String,
    pub userhash: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct ErrorBody {
    pub success: bool,
    pub error: String,
}

pub enum IdentityResult {
    Identified(IdentityBody),
    NotIdentified(ErrorBody),
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct World {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub state: State,
    pub is_wip: Option<bool>,
    pub is_draft: Option<bool>,
    pub entity_class: String,
}

#[derive(Deserialize)]
pub struct WorldsForUserResponse {
    pub success: bool,
    pub entities: Vec<World>,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn test_article() {
        let article_json = r#"
        {
            "id": "4736f4c7-8cba-4668-ba9b-bc5d0478efe9",
            "title": "04",
            "slug": "04-article",
            "state": "public",
            "isWip": false,
            "isDraft": false,
            "entityClass": "Article",
            "icon": "fas fa-image fa-fw",
            "url": "https://www.worldanvil.com/w/solaris-nnie/a/04-article",
            "subscribergroups": [],
            "folderId": "cb66dac7-5818-440a-b6cf-5797d2c20729",
            "tags": "2022-jul",
            "updateDate": {
                "date": "2024-09-02 10:48:03.000000",
                "timezone_type": 3,
                "timezone": "Europe/London"
            },
            "position": null
        }"#;
        let article = serde_json::from_str::<Article>(article_json).unwrap();
        assert_eq!(
            article,
            Article {
                id: "4736f4c7-8cba-4668-ba9b-bc5d0478efe9".to_string(),
                title: "04".to_string(),
                slug: "04-article".to_string(),
                state: State::Public,
                is_wip: false,
                is_draft: false,
                entity_class: EntityClass::Article,
                icon: "fas fa-image fa-fw".to_string(),
                url: "https://www.worldanvil.com/w/solaris-nnie/a/04-article".to_string(),
                folder_id: "cb66dac7-5818-440a-b6cf-5797d2c20729".to_string(),
                tags: Some("2022-jul".to_string()),
                update_date: Date {
                    date: "2024-09-02 10:48:03.000000".to_string(),
                    timezone_type: 3,
                    timezone: "Europe/London".to_string(),
                },
                subscribergroups: vec![],
                position: None,
            }
        )
    }

    #[test]
    fn test_limit_offset() {
        let json = serde_json::to_string(&LimitOffsetBody {
            limit: 50,
            offset: 26,
        })
        .unwrap();
        assert_eq!(json, r#"{"limit":"50","offset":"26"}"#)
    }

    #[test]
    fn test_user_identity() {
        let json = r#"
{
  "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
  "success": true,
  "username": "Username",
  "userhash": "userhash"
}
        "#;
        let value: IdentityBody = serde_json::from_str(&json).unwrap();
        assert_eq!(
            value,
            IdentityBody {
                id: "3fa85f64-5717-4562-b3fc-2c963f66afa6".to_string(),
                success: true,
                username: "Username".to_string(),
                userhash: "userhash".to_string(),
            }
        );
    }
}
