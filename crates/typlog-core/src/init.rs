use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::scaffold::ensure_scaffold_templates;

pub fn init_workspace(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("无法创建目录: {}", dir.display()))?;
    for child in ["post", "templates", "public/posts", "public/assets"] {
        let target = dir.join(child);
        fs::create_dir_all(&target)
            .with_context(|| format!("无法创建目录: {}", target.display()))?;
    }

    let config_path = dir.join("config.toml");
    if !config_path.exists() {
        let config = r#"title = "Typlog"
base_url = "/"
language = "zh-CN"
"#;
        fs::write(&config_path, config)
            .with_context(|| format!("无法写入文件: {}", config_path.display()))?;
    }

    ensure_scaffold_templates(&dir.join("templates"))?;

    println!("初始化完成: {}", dir.display());
    Ok(())
}
