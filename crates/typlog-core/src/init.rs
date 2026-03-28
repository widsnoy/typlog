use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

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

    let article_tpl = dir.join("templates/article.typ");
    if !article_tpl.exists() {
        let template = r#"// 文章入口模板：在 post/<slug>/index.typ 里 import 后调用 article。
// 图片等资源放在与 index.typ 同一目录，正文中使用 #image("foo.png")。
//
// 用法：#article("标题", "2026-03-27")[ 正文… ]

#let article(title, date, body) = {
  set document(title: title)
  [= #title]
  text(size: 0.9em, fill: gray)[日期：#date]
  parbreak()
  body
}
"#;
        fs::write(&article_tpl, template)
            .with_context(|| format!("无法写入文件: {}", article_tpl.display()))?;
    }

    println!("初始化完成: {}", dir.display());
    Ok(())
}
