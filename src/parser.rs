use crate::req::get_default_reqwest;
use anyhow;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

pub struct Article {
    pub title: String,
    pub worldanvil_id: String,
    pub world_worldanvil_id: String,
    pub comments: Vec<RootComment>,
}

#[derive(Debug)]
pub struct RootComment {
    pub comment: Comment,
    pub author_worldanvil_id: String,
    pub replies: Vec<Comment>,
}

#[derive(Debug)]
pub struct Comment {
    pub index: i16,
    pub author_avatar: String,
    pub author_name: String,
    pub comment_datetime: String,
    pub content: String,
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
    let comment_datetime = get_text_content(
        &element
            .select(&get_selector(".comment-box-date"))
            .next()
            .expect("Could not find comment-box-date"),
    );
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

pub fn parse_page(page_body: &str) -> Article {
    let page = Html::parse_document(page_body);
    let world_node = page
        .select(&get_selector("#visual-container"))
        .next()
        .expect("Could not find #visual-container");
    let world_worldanvil_id = find_class_with_prefix(&world_node, "world");
    let title_node = page
        .select(&get_selector("#content .article-title h1"))
        .next()
        .expect("Could not find '#content .article-title h1' attribute");
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
        .expect("Could not find .page-article-main attribute");
    let worldanvil_id = find_class_with_prefix(&article_node, "article");
    // Handle all comments and their replies.
    let mut comments = vec![];
    for (index, element) in page.select(&get_selector(".comment-box")).enumerate() {
        let comment = get_comment_info(&element, index as i16);
        comments.push(RootComment {
            comment,
            author_worldanvil_id: "".to_string(),
            replies: vec![],
        });
    }

    Article {
        title,
        worldanvil_id,
        world_worldanvil_id,
        comments,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

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
        println!("{:#?}", article.comments);
    }
}
