use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use chrono::Local;

use crate::scaffold::{
    DEFAULT_META_TOML_TEMPLATE, DEFAULT_POST_TYP_TEMPLATE, load_template_or_default,
    render_template,
};

/// 校验文章目录名（用作 `posts/<id>/` 与 URL 路径段）：小写字母、数字、短横线。
pub fn validate_post_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("文章 id 不能为空");
    }
    let is_valid = id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !is_valid {
        bail!("文章 id 仅允许小写字母、数字、短横线");
    }
    Ok(())
}

pub fn new_post(id: &str) -> Result<()> {
    validate_post_id(id)?;
    let post_root = Path::new("posts");
    fs::create_dir_all(post_root)
        .with_context(|| format!("无法创建目录: {}", post_root.display()))?;

    let dir = post_root.join(id);
    if dir.exists() {
        bail!("文章目录已存在: {}", dir.display());
    }
    fs::create_dir_all(&dir).with_context(|| format!("无法创建目录: {}", dir.display()))?;

    let today = Local::now().format("%Y-%m-%d").to_string();
    let title = id;

    let meta_tpl =
        load_template_or_default(Path::new("templates/meta.toml"), DEFAULT_META_TOML_TEMPLATE)?;
    let post_tpl =
        load_template_or_default(Path::new("templates/post.typ"), DEFAULT_POST_TYP_TEMPLATE)?;
    let meta_content = render_template(meta_tpl.trim(), &today, title);
    let post_content = post_tpl.trim().to_string();

    let meta_path = dir.join("meta.toml");
    fs::write(&meta_path, meta_content)
        .with_context(|| format!("无法写入文件: {}", meta_path.display()))?;

    let post_path = dir.join("index.typ");
    fs::write(&post_path, post_content)
        .with_context(|| format!("无法写入文件: {}", post_path.display()))?;
    println!("已创建: {} 与 {}", meta_path.display(), post_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_post_id;

    #[test]
    fn post_id_passes_when_kebab_case() {
        assert!(validate_post_id("hello-2026").is_ok());
    }

    #[test]
    fn post_id_fails_when_has_uppercase() {
        assert!(validate_post_id("Hello").is_err());
    }
}
