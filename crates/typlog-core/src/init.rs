use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::default_site_config_toml;
use crate::scaffold::ensure_scaffold_templates;

/// 内置 Material 主题 CSS（与仓库 `themes/material/assets/site.css` 同步，供 `typlog init` 写入工作区）
const MATERIAL_THEME_CSS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../themes/material/assets/site.css"
));

pub fn init_workspace(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("无法创建目录: {}", dir.display()))?;
    for child in ["post", "templates", "public/posts", "public/assets"] {
        let target = dir.join(child);
        fs::create_dir_all(&target)
            .with_context(|| format!("无法创建目录: {}", target.display()))?;
    }
    let theme_assets = dir.join("themes").join("material").join("assets");
    fs::create_dir_all(&theme_assets)
        .with_context(|| format!("无法创建目录: {}", theme_assets.display()))?;

    let config_path = dir.join("config.toml");
    if !config_path.exists() {
        let config = default_site_config_toml().context("序列化默认 config.toml 失败")?;
        fs::write(&config_path, config)
            .with_context(|| format!("无法写入文件: {}", config_path.display()))?;
    }

    ensure_scaffold_templates(&dir.join("templates"))?;

    let material_css = theme_assets.join("site.css");
    if !material_css.exists() {
        fs::write(&material_css, MATERIAL_THEME_CSS).with_context(|| {
            format!("无法写入主题样式 {}", material_css.display())
        })?;
    }

    println!("初始化完成: {}", dir.display());
    Ok(())
}
