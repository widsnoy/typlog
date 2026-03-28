//! 新建文章时使用的模板：`templates/meta.toml` 与 `templates/post.typ`（占位符见模块内常量说明）。

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// 默认 `templates/meta.toml` 内容（含占位符，未替换前）。
pub const DEFAULT_META_TOML_TEMPLATE: &str = r#"title = "{title}"
date = "{date}"
draft = false
"#;

/// 默认 `templates/post.typ` 内容；生成到文章目录时写入为 `index.typ`。
pub const DEFAULT_POST_TYP_TEMPLATE: &str = r#"#import "/templates/article.typ": article

#article("{title}", "{date}")[
在这里开始写正文。
]
"#;

/// 占位符：`{slug}`、`{date}`、`{title}`（新建时 `title` 默认等于 slug，可在 meta.toml 里改）。
pub fn render_template(template: &str, slug: &str, date: &str, title: &str) -> String {
    template
        .replace("{slug}", slug)
        .replace("{date}", date)
        .replace("{title}", title)
}

/// 读取模板文件；不存在或仅空白时返回 `default_template`。
pub fn load_template_or_default(path: &Path, default_template: &str) -> Result<String> {
    if !path.is_file() {
        return Ok(default_template.to_string());
    }
    let s = fs::read_to_string(path).with_context(|| format!("无法读取模板 {}", path.display()))?;
    let t = s.trim();
    if t.is_empty() {
        Ok(default_template.to_string())
    } else {
        Ok(s)
    }
}

/// 初始化或补全 `templates/` 下的 `meta.toml`、`post.typ`（不存在或为空时写入内置默认）。
pub fn ensure_scaffold_templates(templates_dir: &Path) -> Result<()> {
    fs::create_dir_all(templates_dir)
        .with_context(|| format!("无法创建目录: {}", templates_dir.display()))?;

    let meta_path = templates_dir.join("meta.toml");
    if should_seed(&meta_path)? {
        fs::write(&meta_path, DEFAULT_META_TOML_TEMPLATE)
            .with_context(|| format!("无法写入 {}", meta_path.display()))?;
    }

    let post_path = templates_dir.join("post.typ");
    if should_seed(&post_path)? {
        fs::write(&post_path, DEFAULT_POST_TYP_TEMPLATE)
            .with_context(|| format!("无法写入 {}", post_path.display()))?;
    }

    Ok(())
}

fn should_seed(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(true);
    }
    let s = fs::read_to_string(path).with_context(|| format!("无法读取 {}", path.display()))?;
    Ok(s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::render_template;

    #[test]
    fn render_replaces_placeholders() {
        let out = render_template("x {slug} {date} {title}", "a", "2026-01-01", "T");
        assert_eq!(out, "x a 2026-01-01 T");
    }
}
