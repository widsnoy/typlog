use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use chrono::Local;

pub fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        bail!("slug 不能为空");
    }
    let is_valid = slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !is_valid {
        bail!("slug 仅允许小写字母、数字、短横线");
    }
    Ok(())
}

pub fn new_post(slug: &str) -> Result<()> {
    validate_slug(slug)?;
    let post_root = Path::new("post");
    fs::create_dir_all(post_root)
        .with_context(|| format!("无法创建目录: {}", post_root.display()))?;

    let dir = post_root.join(slug);
    if dir.exists() {
        bail!("文章目录已存在: {}", dir.display());
    }
    fs::create_dir_all(&dir).with_context(|| format!("无法创建目录: {}", dir.display()))?;

    let post_path = dir.join("index.typ");
    let today = Local::now().format("%Y-%m-%d").to_string();
    let content = format!(
        r#"#import "/templates/article.typ": article

#article("{slug}", "{today}")[
在这里开始写正文。
]
"#,
        slug = slug,
        today = today,
    );
    fs::write(&post_path, content)
        .with_context(|| format!("无法写入文件: {}", post_path.display()))?;
    println!("已创建: {}", post_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_slug;

    #[test]
    fn slug_should_pass_when_kebab_case() {
        assert!(validate_slug("hello-2026").is_ok());
    }

    #[test]
    fn slug_should_fail_when_has_uppercase() {
        assert!(validate_slug("Hello").is_err());
    }
}
