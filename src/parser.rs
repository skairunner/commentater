use crate::db::schema::{CommentInsert, WorldAnvilUserInsert};
use crate::req::get_default_reqwest;
use anyhow;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use time::macros::format_description;
use time::{error::Parse as TimeParseError, PrimitiveDateTime, UtcOffset};

pub struct Article {
    pub title: String,
    pub worldanvil_id: String,
    pub world_worldanvil_id: String,
    pub comments: Vec<RootComment>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("could not find #visual-container")]
    NoVisualContainer,
    #[error("could not find #content .article-title h1")]
    NoHeader,
    #[error("could not find .page-article-main")]
    NoPageArticleMain,
}

#[derive(Debug)]
pub struct RootComment {
    pub comment: Comment,
    pub author_worldanvil_id: String,
    pub replies: Vec<Comment>,
}

impl RootComment {
    pub fn datetime(&self) -> &PrimitiveDateTime {
        &self.comment.comment_datetime
    }

    pub fn content(&self) -> &str {
        &self.comment.content
    }

    pub fn as_worldanvil_user(&self) -> WorldAnvilUserInsert {
        WorldAnvilUserInsert {
            worldanvil_id: Some(self.author_worldanvil_id.clone()),
            name: self.comment.author_name.clone(),
            avatar_url: Some(self.comment.author_avatar.clone()),
        }
    }
}

#[derive(Debug)]
pub struct Comment {
    pub index: i16,
    pub author_avatar: String,
    pub author_name: String,
    pub comment_datetime: PrimitiveDateTime,
    pub content: String,
}

impl Comment {
    pub fn as_db_comment(
        &self,
        user_id: i64,
        author_id: i64,
        article_id: i64,
        offset: UtcOffset,
    ) -> CommentInsert {
        CommentInsert {
            user_id,
            author_id,
            article_id,
            content: self.content.clone(),
            date: self.comment_datetime.assume_offset(offset),
        }
    }
}

pub async fn get_page(url: &str) -> anyhow::Result<String> {
    let req = get_default_reqwest();
    let r = req.get(url).send().await?;
    Ok(r.text().await?)
}

/// Shorthand to make a selector.
fn get_selector(s: &str) -> Selector {
    Selector::parse(s).unwrap()
}

lazy_static! {
    static ref WORLDANVIL_ID_PATTERN: Regex =
        Regex::new(r#"^([\w-]+)-(\w{8}-\w{4}-\w{4}-\w{4}-\w{12})$"#).unwrap();
}

/// Get the text contents of the node while trimming extra whitespace.
fn get_text_content(element: &ElementRef) -> String {
    element.text().map(str::trim).intersperse("\n").collect()
}

/// Find the class of the element that matches the pattern.
/// Return the element's 1-th group
fn find_class_with_prefix(element: &ElementRef, prefix: &str) -> String {
    let classes = element
        .attr("class")
        .expect("Could not find 'class' attribute on #visual-container");
    for class in classes.split(" ") {
        if let Some(capture) = WORLDANVIL_ID_PATTERN.captures(class) {
            if capture.get(1).unwrap().as_str() != prefix {
                continue;
            }
            return capture.get(2).unwrap().as_str().to_string();
        }
    }
    panic!(
        "{}",
        format!("Could not find a class starting with '{prefix}'")
    )
}

/// Extract common comment info from a node
fn get_comment_info(element: &ElementRef, index: i16) -> Comment {
    let author_name = get_text_content(
        &element
            .select(&get_selector("span.uss-css-user-username"))
            .next()
            .expect("Could not find .uss-css-user-username"),
    );
    let author_avatar = element
        .select(&get_selector("div.comment-box-avatar .img-avatar"))
        .next()
        .expect("Could not find .comment-box-avatar")
        .attr("src")
        .expect("Could not find attribute 'src' on img")
        .to_string();
    let datetime_str = element
        .select(&get_selector(".comment-box-date"))
        .next()
        .expect("Could not find comment-box-date")
        .text()
        .next()
        .unwrap()
        .trim()
        .to_string();
    let comment_datetime =
        parse_date(&datetime_str).unwrap_or_else(|_| panic!("Invalid date {datetime_str}"));
    let content = get_text_content(
        &element
            .select(&get_selector(".comment-box-content p"))
            .next()
            .expect("Could not find .comment-box-content"),
    );
    Comment {
        index,
        author_avatar,
        author_name,
        comment_datetime,
        content,
    }
}

/// Parse an article page and extract all information we need from it.
pub fn parse_page(page_body: &str) -> Result<Article, ParseError> {
    let page = Html::parse_document(page_body);
    let world_node = page
        .select(&get_selector("#visual-container"))
        .next()
        .ok_or(ParseError::NoVisualContainer)?;
    let world_worldanvil_id = find_class_with_prefix(&world_node, "world");
    let title_node = page
        .select(&get_selector("#content .article-title h1"))
        .next()
        .ok_or(ParseError::NoHeader)?;
    // Find all the text nodes, then join and split
    let title = title_node
        .text()
        .intersperse("")
        .collect::<String>()
        .trim()
        .to_string();
    let article_node = page
        .select(&get_selector(".page-article-main"))
        .next()
        .ok_or(ParseError::NoPageArticleMain)?;
    let worldanvil_id = find_class_with_prefix(&article_node, "article");
    // Handle all comments and their replies.
    let mut comments = vec![];
    for (index, element) in page.select(&get_selector(".comment-box")).enumerate() {
        let comment = get_comment_info(&element, index as i16);
        let mut replies = vec![];
        for (index, reply) in element
            .select(&get_selector(".comment-box-reply"))
            .enumerate()
        {
            replies.push(get_comment_info(&reply, index as i16));
        }
        let author_worldanvil_id = find_class_with_prefix(&element, "comment-author");
        comments.push(RootComment {
            comment,
            author_worldanvil_id,
            replies,
        });
    }

    Ok(Article {
        title,
        worldanvil_id,
        world_worldanvil_id,
        comments,
    })
}

pub fn parse_date(s: &str) -> Result<PrimitiveDateTime, TimeParseError> {
    let format = format_description!(
        "[month repr:short] [day padding:none], [year] [hour repr:24]:[minute]"
    );
    PrimitiveDateTime::parse(s, format)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use time::macros::datetime;

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("Aug 24, 2024 03:12"),
            Ok(datetime!(2024-08-24 03:12))
        )
    }

    #[test]
    fn test_parse_page() {
        let fixture = fs::read_to_string("fixtures/example-solaris-page.htm").unwrap();
        let article = parse_page(&fixture);
        assert_eq!(article.title, "Chewpaper");
        assert_eq!(
            article.world_worldanvil_id,
            "e69d6a36-2d22-4bf2-80f9-456a9b0d909e"
        );
        assert_eq!(
            article.worldanvil_id,
            "4cdfec2c-b875-4dc6-b5c9-146470e9ac80"
        );
        let expected_authors = vec![
            ("Tyrdal", "d05d748e-57d9-42f6-80fc-eff50fabda50"),
            ("CoolG1319", "b51561d7-f49f-4493-85b1-5f5b2ff4c243"),
            ("skairunner", "9fe45c42-cb7e-47f0-bfb0-bd98762dda16"),
        ];
        for ((expected_author, expected_id), comment) in
            expected_authors.iter().zip(&article.comments)
        {
            assert_eq!(*expected_author, comment.comment.author_name);
            assert_eq!(*expected_id, comment.author_worldanvil_id);
            assert_eq!(comment.replies.len(), 1);
            assert_eq!(comment.replies[0].author_name, "nnie");
        }
        println!("{:#?}", article.comments);
    }
}
