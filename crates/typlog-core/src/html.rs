use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::SiteConfig;
use crate::meta::PostMeta;

const INDEX_HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/index.html"));
const INDEX_POST_ITEM_HTML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/index_post_item.html"));

pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn theme_css_path_index(theme: &str) -> String {
    format!("assets/themes/{theme}/site.css")
}

fn theme_css_path_post(theme: &str) -> String {
    format!("../../assets/themes/{theme}/site.css")
}

/// `path_prefix`：相对当前 HTML 文件到站点根，首页为 `""`，文章为 `"../../"`。
fn resolve_background_image_url(raw: &str, path_prefix: &str) -> String {
    let s = raw.trim();
    if s.starts_with("http://") || s.starts_with("https://") {
        return s.to_string();
    }
    let path = s.trim_start_matches('/');
    format!("{path_prefix}{path}")
}

fn css_url_value(url: &str) -> String {
    let escaped = url.replace('\\', "\\\\").replace('"', "\\\"");
    format!("url(\"{escaped}\")")
}

/// 根据 `config.toml` 注入全站背景图 / 透明度 / 模糊（`body::before` 固定层）。
pub(crate) fn background_style_for_site(site: &SiteConfig, path_prefix: &str) -> String {
    let Some(img) = site.background_image.as_ref() else {
        return String::new();
    };
    if img.trim().is_empty() {
        return String::new();
    }
    let url = resolve_background_image_url(img, path_prefix);
    let opacity = site.background_opacity.clamp(0.0, 1.0);
    let blur = site.background_blur_px;
    let url_css = css_url_value(&url);
    format!(
        r#"<style id="typlog-site-bg">
body.typlog-material {{ background-color: transparent !important; }}
body.typlog-material::before {{
  content: "";
  display: block;
  position: fixed;
  inset: 0;
  z-index: -1;
  pointer-events: none;
  background-image: {url_css};
  background-size: cover;
  background-position: center;
  opacity: {opacity};
  filter: blur({blur}px);
  transform: scale(1.06);
}}
</style>
"#,
        url_css = url_css,
        opacity = opacity,
        blur = blur,
    )
}

fn render_index_post_items(posts: &[PostMeta]) -> String {
    let mut out = String::new();
    for p in posts {
        let date_html = match p.date {
            Some(d) => {
                let iso = d.format("%Y-%m-%d").to_string();
                format!(r#"<time datetime="{iso}">{iso}</time>"#)
            }
            None => String::new(),
        };
        let row = INDEX_POST_ITEM_HTML
            .replace("{{typlog_post_id}}", &html_escape(&p.id))
            .replace("{{typlog_post_title}}", &html_escape(&p.title))
            .replace("{{typlog_post_date}}", &date_html);
        out.push_str(&row);
    }
    out
}

fn render_index_html(site: &SiteConfig, posts: &[PostMeta]) -> String {
    let css_href = html_escape(&theme_css_path_index(site.theme.as_str()));
    let title = html_escape(site.title.as_str());
    let lang = html_escape(site.language.as_str());
    let post_items = render_index_post_items(posts);
    INDEX_HTML
        .replace("{{typlog_lang}}", &lang)
        .replace("{{typlog_title}}", &title)
        .replace("{{typlog_css_href}}", &css_href)
        .replace(
            "{{typlog_background_style}}",
            &background_style_for_site(site, ""),
        )
        .replace("{{typlog_post_items}}", &post_items)
}

/// 生成首页 HTML（Material 风格壳 + 主题 CSS）；结构见 `templates/index.html`。
pub fn write_index_html(out: &Path, site: &SiteConfig, posts: &[PostMeta]) -> Result<()> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {}", parent.display()))?;
    }
    let html = render_index_html(site, posts);
    fs::write(out, html).with_context(|| format!("无法写入 {}", out.display()))?;
    Ok(())
}

/// 在 Typst 生成的文章 HTML 上注入 Material 顶栏、文首 meta 与主题样式，并把正文包进 `.material-typst`。
pub fn inject_theme_post_html(html: &str, site: &SiteConfig, meta: &PostMeta) -> String {
    let theme = site.theme.as_str();
    let css_href = theme_css_path_post(theme);
    let home = "../../index.html";
    let site_title = html_escape(site.title.as_str());
    let post_title = html_escape(meta.title.as_str());
    let date_line = match meta.date {
        Some(d) => {
            let iso = d.format("%Y-%m-%d").to_string();
            format!(
                "<p class=\"material-article-date\"><time datetime=\"{iso}\">{iso}</time></p>\n"
            )
        }
        None => String::new(),
    };

    let mut s = html.to_string();
    let head_snippet = format!(
        r#"    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="{css_href}">
"#
    );
    let bg = background_style_for_site(site, "../../");
    let mut inject = String::new();
    if !s.contains(&css_href) {
        inject.push_str(&head_snippet);
    }
    if !bg.is_empty() && !s.contains("typlog-site-bg") {
        inject.push_str(&bg);
    }
    if !inject.is_empty()
        && let Some(pos) = s.to_ascii_lowercase().find("</head>")
    {
        s.insert_str(pos, &inject);
    }

    let lower = s.to_ascii_lowercase();
    let Some(body_tag_start) = lower.find("<body") else {
        return s;
    };
    let after = &s[body_tag_start..];
    let Some(gt_rel) = after.find('>') else {
        return s;
    };
    let inner_start = body_tag_start + gt_rel + 1;
    let Some(body_end) = lower.rfind("</body>") else {
        return s;
    };
    if body_end <= inner_start {
        return s;
    }
    let inner = s[inner_start..body_end].trim();

    let mut hero = String::new();
    hero.push_str("<header class=\"material-appbar\"><div class=\"material-appbar-inner\"><a class=\"material-appbar-brand\" href=\"");
    hero.push_str(home);
    hero.push_str("\">");
    hero.push_str(&site_title);
    hero.push_str("</a></div></header>\n<main class=\"material-post-main\">\n<section class=\"material-article-hero\">\n<h1 class=\"material-article-title\">");
    hero.push_str(&post_title);
    hero.push_str("</h1>\n");
    hero.push_str(&date_line);
    hero.push_str("</section>\n<div class=\"material-typst\">\n");
    hero.push_str(inner);
    hero.push_str("\n</div>\n</main>");

    let before = &s[..body_tag_start];
    let tail = &s[body_end + "</body>".len()..];
    let mut out = String::with_capacity(before.len() + hero.len() + tail.len() + 64);
    out.push_str(before);
    out.push_str("<body class=\"typlog-material\">\n");
    out.push_str(&hero);
    out.push_str("\n</body>");
    out.push_str(tail);
    out
}

#[cfg(test)]
mod tests {
    use super::{html_escape, inject_theme_post_html};
    use crate::config::SiteConfig;
    use crate::meta::PostMeta;
    use chrono::NaiveDate;

    #[test]
    fn html_escape_should_escape_special_chars() {
        assert_eq!(html_escape("&<>"), "&amp;&lt;&gt;");
        assert_eq!(html_escape("\""), "&quot;");
    }

    #[test]
    fn inject_post_wraps_body_and_keeps_doctype() {
        let site = SiteConfig {
            title: "Blog".to_string(),
            ..Default::default()
        };
        let meta = PostMeta {
            id: "a".into(),
            title: "Title".into(),
            date: Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()),
            updated: None,
            draft: false,
        };
        let raw = "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>x</title>\n</head>\n<body>\n<p>Hi</p>\n</body>\n</html>\n";
        let out = inject_theme_post_html(raw, &site, &meta);
        assert!(out.contains("material-appbar"));
        assert!(out.contains("material-typst"));
        assert!(out.contains("<p>Hi</p>"));
        assert!(out.contains("assets/themes/material/site.css"));
    }

    #[test]
    fn background_style_resolves_post_relative_path() {
        let site = SiteConfig {
            background_image: Some("assets/bg.jpg".into()),
            background_opacity: 0.7,
            background_blur_px: 8,
            ..Default::default()
        };
        let s = super::background_style_for_site(&site, "../../");
        assert!(s.contains("typlog-site-bg"));
        assert!(s.contains("../../assets/bg.jpg"));
        assert!(s.contains("opacity: 0.7"));
        assert!(s.contains("blur(8px)"));
    }
}
