use std::fs;
use std::path::Path;

use serde::Deserialize;

/// 站点级配置（`config.toml`），与文章 `meta.toml` 分离。
#[derive(Debug, Clone, Deserialize)]
pub struct SiteConfig {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_title() -> String {
    "Typlog".to_string()
}

fn default_base_url() -> String {
    "/".to_string()
}

fn default_language() -> String {
    "zh-CN".to_string()
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: default_title(),
            base_url: default_base_url(),
            language: default_language(),
        }
    }
}

/// 读取仓库根目录的 `config.toml`；文件缺失或解析失败时返回 [`SiteConfig::default`]。
pub fn load_site_config() -> SiteConfig {
    load_site_config_from_path(Path::new("config.toml"))
}

/// 从给定路径加载；便于测试或自定义工作目录。
pub fn load_site_config_from_path(path: &Path) -> SiteConfig {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn parses_full_config() {
        let dir = std::env::temp_dir();
        let p = dir.join("typlog-site-config-test.toml");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(
            br#"title = "Blog"
base_url = "https://example.com/"
language = "en"
"#,
        )
        .unwrap();
        drop(f);
        let c = load_site_config_from_path(&p);
        assert_eq!(c.title, "Blog");
        assert_eq!(c.base_url, "https://example.com/");
        assert_eq!(c.language, "en");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn missing_file_yields_default() {
        let p = std::env::temp_dir().join("typlog-no-config-xyz-99999.toml");
        let _ = std::fs::remove_file(&p);
        let c = load_site_config_from_path(&p);
        assert_eq!(c.title, "Typlog");
        assert_eq!(c.base_url, "/");
        assert_eq!(c.language, "zh-CN");
    }

    #[test]
    fn partial_config_fills_field_defaults() {
        let dir = std::env::temp_dir();
        let p = dir.join("typlog-partial-config-test.toml");
        std::fs::write(&p, r#"title = "Only""#).unwrap();
        let c = load_site_config_from_path(&p);
        assert_eq!(c.title, "Only");
        assert_eq!(c.base_url, "/");
        assert_eq!(c.language, "zh-CN");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn invalid_toml_yields_default() {
        let dir = std::env::temp_dir();
        let p = dir.join("typlog-bad-config-test.toml");
        std::fs::write(&p, "not toml {{{").unwrap();
        let c = load_site_config_from_path(&p);
        assert_eq!(c.title, "Typlog");
        let _ = std::fs::remove_file(&p);
    }
}
