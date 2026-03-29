use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::default_site_config_toml;
use crate::scaffold::ensure_scaffold_templates;

/// 内置主题 CSS（与仓库 `themes/material/site.css` 同步，供 `typlog init` 写入工作区）
const DEFAULT_THEME_CSS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../themes/material/site.css"
));

/// Hotwired Turbo UMD（与仓库 `themes/material/turbo.min.js` 同步）
const DEFAULT_TURBO_JS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../themes/material/turbo.min.js"
));

pub fn init_workspace(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("无法创建目录: {}", dir.display()))?;
    for child in ["posts", "templates"] {
        let target = dir.join(child);
        fs::create_dir_all(&target)
            .with_context(|| format!("无法创建目录: {}", target.display()))?;
    }
    let theme_dir = dir.join("themes").join("material");
    fs::create_dir_all(&theme_dir)
        .with_context(|| format!("无法创建目录: {}", theme_dir.display()))?;

    let config_path = dir.join("config.toml");
    if !config_path.exists() {
        let config = default_site_config_toml().context("序列化默认 config.toml 失败")?;
        fs::write(&config_path, config)
            .with_context(|| format!("无法写入文件: {}", config_path.display()))?;
    }

    ensure_scaffold_templates(&dir.join("templates"))?;

    let theme_css = theme_dir.join("site.css");
    if !theme_css.exists() {
        fs::write(&theme_css, DEFAULT_THEME_CSS).with_context(|| {
            format!("无法写入主题样式 {}", theme_css.display())
        })?;
    }

    let turbo_js = theme_dir.join("turbo.min.js");
    if !turbo_js.exists() {
        fs::write(&turbo_js, DEFAULT_TURBO_JS).with_context(|| {
            format!("无法写入 Turbo 脚本 {}", turbo_js.display())
        })?;
    }

    println!("初始化完成: {}", dir.display());
    Ok(())
}
