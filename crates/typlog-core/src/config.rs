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
    /// 全站背景图：相对站点根的路径（如 `background.webp`）或 `https://...`；空字符串表示无背景图。
    /// 若滚动仍卡顿，可换用**预模糊**的静态图（如本地用工具生成 `background-blurred.webp`），并将 `background_blur_px` 设为 `0`，避免运行时对整图做 CSS `filter: blur()`。
    /// 从当前版本起，当 `background_blur_px > 0` 且为本地图片路径时，`typlog generate` 也会自动预处理为 `public/background-preblur.png`。
    #[serde(default)]
    pub background_image: String,
    /// 背景图透明度，0～1（作用于背景图层）
    #[serde(default = "default_background_opacity")]
    pub background_opacity: f64,
    /// 背景模糊（像素），0 表示不模糊
    #[serde(default)]
    pub background_blur_px: u32,
    /// 首页 hero 区签名/标语；空字符串表示不显示 hero 区
    #[serde(default)]
    pub signature: String,
    /// 主题 ID，对应仓库根目录 `themes/<theme_id>/`（`typlog generate` 将该目录内容复制到 `public/` 根）
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Material 主题下首页/文章主栏「白底」不透明度，约 0.35～1；越大越实、越不透出背后 `#typlog-bg`。
    /// 与 `background_opacity`（整张背景图层）无关。
    #[serde(default = "default_glass_panel_opacity")]
    pub glass_panel_opacity: f64,
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

fn default_background_opacity() -> f64 {
    1.0
}

fn default_theme() -> String {
    "material".to_string()
}

fn default_glass_panel_opacity() -> f64 {
    0.78
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: default_title(),
            base_url: default_base_url(),
            language: default_language(),
            background_image: String::new(),
            background_opacity: default_background_opacity(),
            background_blur_px: 0,
            signature: String::new(),
            theme: default_theme(),
            glass_panel_opacity: default_glass_panel_opacity(),
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
        assert_eq!(c.theme, "material");
        assert!((c.glass_panel_opacity - 0.78).abs() < f64::EPSILON);
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
        assert!(c.background_image.is_empty());
        assert_eq!(c.background_opacity, 1.0);
        assert_eq!(c.background_blur_px, 0);
        assert!((c.glass_panel_opacity - 0.78).abs() < f64::EPSILON);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn default_config_roundtrips_toml() {
        let s = default_site_config_toml().expect("serialize");
        let parsed: SiteConfig = toml::from_str(&s).expect("deserialize");
        assert_eq!(parsed, SiteConfig::default());
    }

    #[test]
    fn default_site_config_toml_lists_background_and_signature() {
        let s = default_site_config_toml().expect("serialize");
        assert!(
            s.contains("background_image")
                && s.contains("background_opacity")
                && s.contains("background_blur_px")
                && s.contains("signature")
                && s.contains("theme")
                && s.contains("glass_panel_opacity"),
            "typlog init 写入的 config 应包含背景与签名字段，便于用户直接编辑：\n{s}"
        );
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
