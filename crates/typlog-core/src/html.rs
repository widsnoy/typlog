use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

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

pub fn write_index_html(out: &Path, site_title: &str, posts: &[PostMeta]) -> Result<()> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {}", parent.display()))?;
    }
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<title>");
    html.push_str(&html_escape(site_title));
    html.push_str("</title>\n</head>\n<body>\n");
    html.push_str("<h1>");
    html.push_str(&html_escape(site_title));
    html.push_str("</h1>\n<ul>\n");
    for p in posts {
        html.push_str("<li><a href=\"posts/");
        html.push_str(&p.slug);
        html.push_str("/index.html\">");
        html.push_str(&html_escape(&p.title));
        html.push_str("</a>");
        if let Some(d) = p.date {
            let _ = write!(&mut html, " <span>{}</span>", d.format("%Y-%m-%d"));
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ul>\n</body>\n</html>\n");
    fs::write(out, html).with_context(|| format!("无法写入 {}", out.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::html_escape;

    #[test]
    fn html_escape_should_escape_special_chars() {
        assert_eq!(html_escape("&<>"), "&amp;&lt;&gt;");
        assert_eq!(html_escape("\""), "&quot;");
    }
}
