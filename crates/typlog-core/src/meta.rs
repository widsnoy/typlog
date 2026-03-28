use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;

/// 博客列表与排序用的文章元数据（来自 `post/<slug>/meta.toml`）。
#[derive(Debug, Clone)]
pub struct PostMeta {
    pub slug: String,
    pub title: String,
    pub date: Option<NaiveDate>,
    /// 修改时间，原始字符串（展示或 RSS 用）
    pub updated: Option<String>,
    pub draft: bool,
}

#[derive(Debug, Deserialize)]
struct MetaFile {
    title: String,
    date: String,
    #[serde(default)]
    updated: Option<String>,
    #[serde(default)]
    draft: bool,
}

/// 从 `post/<slug>/` 目录读取 `meta.toml`。
pub fn post_meta_from_post_dir(post_dir: &Path) -> Result<PostMeta> {
    let slug = post_dir
        .file_name()
        .and_then(|s| s.to_str())
        .context("无法从路径解析 slug")?
        .to_string();
    let path = post_dir.join("meta.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("无法读取 {}", path.display()))?;
    let m: MetaFile =
        toml::from_str(&raw).with_context(|| format!("解析 meta.toml 失败: {}", path.display()))?;
    let date = NaiveDate::parse_from_str(m.date.trim(), "%Y-%m-%d")
        .with_context(|| format!("meta.toml 中 date 须为 YYYY-MM-DD，实际: {}", m.date))?;
    Ok(PostMeta {
        slug,
        title: m.title,
        date: Some(date),
        updated: m.updated,
        draft: m.draft,
    })
}

pub fn sort_posts_desc(posts: &mut [PostMeta]) {
    posts.sort_by(|a, b| match (&a.date, &b.date) {
        (Some(da), Some(db)) => db.cmp(da),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.slug.cmp(&b.slug),
    });
}

#[cfg(test)]
mod tests {
    use super::MetaFile;

    #[test]
    fn meta_toml_deserializes() {
        let raw = r#"
title = "Hello"
date = "2026-03-01"
updated = "2026-03-02"
draft = true
"#;
        let m: MetaFile = toml::from_str(raw).unwrap();
        assert_eq!(m.title, "Hello");
        assert_eq!(m.date, "2026-03-01");
        assert_eq!(m.updated.as_deref(), Some("2026-03-02"));
        assert!(m.draft);
    }

    #[test]
    fn meta_toml_draft_defaults_false() {
        let raw = r#"
title = "X"
date = "2026-01-01"
"#;
        let m: MetaFile = toml::from_str(raw).unwrap();
        assert!(!m.draft);
    }
}
