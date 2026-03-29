use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::SiteConfig;
use crate::meta::PostMeta;

const INDEX_HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/index.html"));
const INDEX_POST_ITEM_HTML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/index_post_item.html"));

/// 首页：主题 CSS、Turbo（与 `themes/material/` 同步到 `public/` 根目录）。
const THEME_CSS_HREF_INDEX: &str = "site.css";
const THEME_TURBO_HREF_INDEX: &str = "turbo.min.js";
/// 文章页相对 `public/posts/<id>/index.html` 的资源路径。
const THEME_CSS_HREF_POST: &str = "../../site.css";
const THEME_TURBO_HREF_POST: &str = "../../turbo.min.js";

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

/// 站点根下的页面或静态资源 URL（`rel` 无前导 `/`，如 `index.html`、`posts/foo/index.html`）。
/// 用于 Turbo 导航与 `data-turbo-permanent` 背景层，避免相对路径随目录变化。
pub(crate) fn site_href(site: &SiteConfig, rel: &str) -> String {
    let rel = rel.trim().trim_start_matches('/');
    let base = site.base_url.trim();
    if base.is_empty() || base == "/" {
        return if rel.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", rel)
        };
    }
    let b = if base.ends_with('/') {
        base.to_string()
    } else {
        format!("{}/", base)
    };
    format!("{b}{rel}")
}

/// 背景图等站点根资源：支持 `https://...`，否则与 [`site_href`] 相同。
fn site_root_asset_url(site: &SiteConfig, raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        return s.to_string();
    }
    site_href(site, s)
}

fn css_url_value(url: &str) -> String {
    let escaped = url.replace('\\', "\\\\").replace('"', "\\\"");
    format!("url(\"{escaped}\")")
}

/// 将 `config.toml` 的 `glass_panel_opacity` 写成 `:root` 变量，覆盖主题 `site.css` 中的默认值。
pub(crate) fn glass_panel_css_vars(site: &SiteConfig) -> String {
    let base = site.glass_panel_opacity.clamp(0.35, 1.0);
    let mid = (base - 0.04).max(0.35);
    let bottom = (base - 0.02).max(0.35);
    format!(
        r#"<style id="typlog-glass-vars">
:root {{
  --glass-body-a: {a:.4};
  --glass-body-a-mid: {mid:.4};
  --glass-body-a-bottom: {bottom:.4};
}}
</style>
"#,
        a = base,
        mid = mid,
        bottom = bottom,
    )
}

/// 根据 `config.toml` 注入全站背景图 / 透明度 / 模糊（`#typlog-bg` 固定层，与 site.css 配套）。
pub(crate) fn background_style_for_site(site: &SiteConfig) -> String {
    let img = site.background_image.trim();
    if img.is_empty() {
        return String::new();
    }
    let url = site_root_asset_url(site, img);
    let opacity = site.background_opacity.clamp(0.0, 1.0);
    let blur = site.background_blur_px;
    let url_css = css_url_value(&url);
    let filter_block = if blur > 0 {
        format!(
            "  filter: blur({blur}px);\n  transform: translate3d(0, 0, 0) scale(1.06);\n"
        )
    } else {
        "  transform: translate3d(0, 0, 0);\n".to_string()
    };
    let reduced_motion_block = if blur > 0 {
        r#"@media (prefers-reduced-motion: reduce) {
  #typlog-bg { filter: none !important; }
}
"#
    } else {
        ""
    };
    format!(
        r#"<style id="typlog-site-bg">
html {{ background: transparent; }}
body.typlog-material {{ background: transparent !important; }}
#typlog-bg {{
  background-image: {url_css};
  background-size: cover;
  background-position: center;
  opacity: {opacity};
{filter_block}
}}
{reduced_motion_block}</style>
"#,
        url_css = url_css,
        opacity = opacity,
        filter_block = filter_block,
        reduced_motion_block = reduced_motion_block,
    )
}

fn render_index_post_items(site: &SiteConfig, posts: &[PostMeta]) -> String {
    let mut out = String::new();
    for p in posts {
        let date_html = match p.date {
            Some(d) => {
                let iso = d.format("%Y-%m-%d").to_string();
                format!(r#"<time datetime="{iso}">{iso}</time>"#)
            }
            None => String::new(),
        };
        let post_href = html_escape(&site_href(
            site,
            &format!("posts/{}/index.html", p.id),
        ));
        let row = INDEX_POST_ITEM_HTML
            .replace("{{typlog_post_href}}", &post_href)
            .replace("{{typlog_post_id}}", &html_escape(&p.id))
            .replace("{{typlog_post_title}}", &html_escape(&p.title))
            .replace("{{typlog_post_date}}", &date_html);
        out.push_str(&row);
    }
    out
}

fn hero_html(site: &SiteConfig) -> String {
    let sig = site.signature.trim();
    if sig.is_empty() {
        return String::new();
    }
    format!(
        "<div class=\"material-index-hero\"><p class=\"material-index-signature\">{}</p></div>\n",
        html_escape(sig)
    )
}

fn render_index_html(site: &SiteConfig, posts: &[PostMeta]) -> String {
    let css_href = html_escape(THEME_CSS_HREF_INDEX);
    let turbo_href = html_escape(THEME_TURBO_HREF_INDEX);
    let home_href = html_escape(&site_href(site, "index.html"));
    let title = html_escape(site.title.as_str());
    let lang = html_escape(site.language.as_str());
    let post_items = render_index_post_items(site, posts);
    INDEX_HTML
        .replace("{{typlog_lang}}", &lang)
        .replace("{{typlog_title}}", &title)
        .replace("{{typlog_css_href}}", &css_href)
        .replace("{{typlog_turbo_script}}", &turbo_href)
        .replace("{{typlog_home_href}}", &home_href)
        .replace("{{typlog_glass_vars}}", &glass_panel_css_vars(site))
        .replace(
            "{{typlog_background_style}}",
            &background_style_for_site(site),
        )
        .replace("{{typlog_hero}}", &hero_html(site))
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
    let css_href = THEME_CSS_HREF_POST;
    let turbo_href = THEME_TURBO_HREF_POST;
    let home = html_escape(&site_href(site, "index.html"));
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
    <script src="{turbo_href}" defer></script>
"#
    );
    let bg = background_style_for_site(site);
    let glass = glass_panel_css_vars(site);
    let mut inject = String::new();
    if !s.contains(css_href) {
        inject.push_str(&head_snippet);
    } else {
        if !s.contains("turbo.min.js") {
            inject.push_str(&format!(
                r#"    <script src="{turbo_href}" defer></script>
"#
            ));
        }
    }
    if !s.contains("typlog-glass-vars") {
        inject.push_str(&glass);
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
    hero.push_str(
        "<div id=\"typlog-bg\" data-turbo-permanent aria-hidden=\"true\"></div>\n",
    );
    hero.push_str("<header class=\"material-appbar\"><div class=\"material-appbar-inner\"><a class=\"material-appbar-brand\" href=\"");
    hero.push_str(&home);
    hero.push_str("\">");
    hero.push_str(&site_title);
    hero.push_str("</a></div></header>\n<main id=\"typlog-main\" class=\"material-post-main\">\n<section class=\"material-article-hero\">\n<h1 class=\"material-article-title\">");
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
    use super::{html_escape, inject_theme_post_html, site_href};
    use crate::config::SiteConfig;
    use crate::meta::PostMeta;
    use chrono::NaiveDate;

    #[test]
    fn html_escape_should_escape_special_chars() {
        assert_eq!(html_escape("&<>"), "&amp;&lt;&gt;");
        assert_eq!(html_escape("\""), "&quot;");
    }

    #[test]
    fn site_href_root() {
        let site = SiteConfig {
            base_url: "/".to_string(),
            ..Default::default()
        };
        assert_eq!(site_href(&site, "index.html"), "/index.html");
        assert_eq!(site_href(&site, "posts/a/index.html"), "/posts/a/index.html");
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
        assert!(out.contains("site.css"));
        assert!(out.contains("typlog-bg"));
        assert!(out.contains("turbo.min.js"));
        assert!(out.contains("href=\"/index.html\""));
        assert!(out.contains("typlog-glass-vars"));
    }

    #[test]
    fn glass_panel_css_vars_follows_config() {
        let site = SiteConfig {
            glass_panel_opacity: 0.9,
            ..Default::default()
        };
        let s = super::glass_panel_css_vars(&site);
        assert!(s.contains("typlog-glass-vars"));
        assert!(s.contains("--glass-body-a: 0.9000"));
    }

    #[test]
    fn background_style_uses_site_root_url() {
        let site = SiteConfig {
            background_image: "bg.jpg".into(),
            background_opacity: 0.7,
            background_blur_px: 8,
            base_url: "/".to_string(),
            ..Default::default()
        };
        let s = super::background_style_for_site(&site);
        assert!(s.contains("typlog-site-bg"));
        assert!(s.contains("url(\"/bg.jpg\")"));
        assert!(s.contains("opacity: 0.7"));
        assert!(s.contains("blur(8px)"));
    }

    #[test]
    fn background_style_omits_filter_when_blur_zero() {
        let site = SiteConfig {
            background_image: "x.webp".into(),
            background_blur_px: 0,
            base_url: "/".to_string(),
            ..Default::default()
        };
        let s = super::background_style_for_site(&site);
        assert!(!s.contains("filter:"));
        assert!(!s.contains("blur("));
    }
}
