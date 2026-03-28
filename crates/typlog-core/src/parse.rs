/// 解析 `#article("标题", "日期")` 形式（均为位置参数与引号字符串）。
pub fn parse_article_call(content: &str) -> Option<(String, String)> {
    let idx = content.find("#article(")?;
    let mut rest = &content[idx + "#article(".len()..];
    let title = parse_leading_quoted(&mut rest)?;
    rest = rest.trim_start();
    rest = rest.strip_prefix(',')?.trim_start();
    let date = parse_leading_quoted(&mut rest)?;
    Some((title, date))
}

pub fn parse_leading_quoted(rest: &mut &str) -> Option<String> {
    let s = rest.trim_start();
    let s = s.strip_prefix('"')?;
    let end = s.find('"')?;
    let val = s[..end].to_string();
    *rest = &s[end + 1..];
    Some(val)
}

pub fn parse_let_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("#let {key} = \"");
    for line in content.lines().take(128) {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        return Some(rest[..end].to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{parse_article_call, parse_let_value};

    #[test]
    fn parse_let_should_read_title_and_date() {
        let s = r#"#let title = "Hello"
#let date = "2026-01-02"
"#;
        assert_eq!(parse_let_value(s, "title").as_deref(), Some("Hello"));
        assert_eq!(parse_let_value(s, "date").as_deref(), Some("2026-01-02"));
    }

    #[test]
    fn parse_article_call_should_read_title_and_date() {
        let s = r#"#import "/templates/article.typ": article

#article("Hello", "2026-01-02")[
正文
]
"#;
        let (t, d) = parse_article_call(s).expect("article");
        assert_eq!(t, "Hello");
        assert_eq!(d, "2026-01-02");
    }
}
