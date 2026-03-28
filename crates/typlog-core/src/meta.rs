use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::NaiveDate;

use crate::parse::{parse_article_call, parse_let_value};

#[derive(Debug, Clone)]
pub struct PostMeta {
    pub slug: String,
    pub title: String,
    pub date: Option<NaiveDate>,
}

pub fn post_meta_from_file(path: &Path) -> Result<PostMeta> {
    let slug = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .context("无法从路径解析 slug")?
        .to_string();
    let content =
        fs::read_to_string(path).with_context(|| format!("无法读取 {}", path.display()))?;
    let (title, date) = if let Some((t, d)) = parse_article_call(&content) {
        let date = NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok();
        (t, date)
    } else {
        let title = parse_let_value(&content, "title").unwrap_or_else(|| slug.clone());
        let date = parse_let_value(&content, "date")
            .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        (title, date)
    };
    Ok(PostMeta { slug, title, date })
}

pub fn sort_posts_desc(posts: &mut [PostMeta]) {
    posts.sort_by(|a, b| match (&a.date, &b.date) {
        (Some(da), Some(db)) => db.cmp(da),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.slug.cmp(&b.slug),
    });
}
