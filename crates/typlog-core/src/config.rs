use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// 站点级配置（`config.toml`），与文章 `meta.toml` 分离。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SiteConfig {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_language")]
    pub language: String,
    /// 主题 id，对应 `themes/<id>/assets/`（由 `typlog generate` 复制到 `public/assets/themes/<id>/`）
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 全站背景图：相对站点根的路径（如 `assets/bg.jpg`）或 `https://...`；不设置则无背景图
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_image: Option<String>,
    /// 背景图透明度，0～1（作用于背景图层）
    #[serde(default = "default_background_opacity")]
    pub background_opacity: f64,
    /// 背景模糊（像素），0 表示不模糊
    #[serde(default)]
    pub background_blur_px: u32,
    /// 首页 hero 区签名/标语；不设置则不显示 hero 区
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
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

fn default_theme() -> String {
    "material".to_string()
}

fn default_background_opacity() -> f64 {
    1.0
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: default_title(),
            base_url: default_base_url(),
            language: default_language(),
            theme: default_theme(),
            background_image: None,
            background_opacity: default_background_opacity(),
            background_blur_px: 0,
            signature: None,
        }
    }
}

/// 将默认 [`SiteConfig`] 序列化为 TOML 文本（供 `typlog init` 写入 `config.toml`）。
pub fn default_site_config_toml() -> Result<String, toml::ser::Error> {
    toml::to_string(&SiteConfig::default())
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
        assert_eq!(c.theme, "material");
        assert!(c.background_image.is_none());
        assert_eq!(c.background_opacity, 1.0);
        assert_eq!(c.background_blur_px, 0);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn default_config_roundtrips_toml() {
        let s = default_site_config_toml().expect("serialize");
        let parsed: SiteConfig = toml::from_str(&s).expect("deserialize");
        assert_eq!(parsed, SiteConfig::default());
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
