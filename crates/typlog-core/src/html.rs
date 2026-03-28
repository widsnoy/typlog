use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::SiteConfig;
use crate::meta::PostMeta;

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

/// 生成首页 HTML（Material 风格壳 + 主题 CSS）。
pub fn write_index_html(out: &Path, site: &SiteConfig, posts: &[PostMeta]) -> Result<()> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {}", parent.display()))?;
    }
    let css_href = theme_css_path_index(site.theme.as_str());
    let lang = html_escape(site.language.as_str());
    let title = html_escape(site.title.as_str());
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"");
    html.push_str(&lang);
    html.push_str("\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<title>");
    html.push_str(&title);
    html.push_str("</title>\n");
    html.push_str("<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n");
    html.push_str("<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>\n");
    html.push_str("<link href=\"https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&display=swap\" rel=\"stylesheet\">\n");
    html.push_str("<link rel=\"stylesheet\" href=\"");
    html.push_str(&html_escape(&css_href));
    html.push_str("\">\n");
    html.push_str("</head>\n<body class=\"typlog-material\">\n");
    html.push_str("<header class=\"material-appbar\"><div class=\"material-appbar-inner\">");
    html.push_str("<a class=\"material-appbar-brand\" href=\"index.html\">");
    html.push_str(&title);
    html.push_str("</a></div></header>\n");
    html.push_str("<main class=\"material-index-wrap\">\n");
    html.push_str("<h1 class=\"material-page-title\">");
    html.push_str(&title);
    html.push_str("</h1>\n");
    html.push_str("<p class=\"material-page-sub\">文章列表</p>\n");
    html.push_str("<ul class=\"material-post-list\">\n");
    for p in posts {
        html.push_str("<li class=\"material-post-card\"><a class=\"material-post-card-inner\" href=\"posts/");
        html.push_str(&html_escape(&p.id));
        html.push_str("/index.html\">\n");
        html.push_str("<h2 class=\"material-post-card-title\">");
        html.push_str(&html_escape(&p.title));
        html.push_str("</h2>\n");
        html.push_str("<p class=\"material-post-card-meta\">");
        if let Some(d) = p.date {
            html.push_str("<time datetime=\"");
            html.push_str(&d.format("%Y-%m-%d").to_string());
            html.push_str("\">");
            html.push_str(&d.format("%Y-%m-%d").to_string());
            html.push_str("</time>");
        }
        html.push_str("</p>\n</a></li>\n");
    }
    html.push_str("</ul>\n</main>\n</body>\n</html>\n");
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
    if !s.contains(&css_href)
        && let Some(pos) = s.to_ascii_lowercase().find("</head>")
    {
        s.insert_str(pos, &head_snippet);
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
}
