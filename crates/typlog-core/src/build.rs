use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config::load_site_config;
use crate::html::{inject_theme_post_html, write_index_html};
use crate::meta::{PostMeta, post_meta_from_post_dir, sort_posts_desc};

pub fn generate(clean: bool, verbose: bool) -> Result<()> {
    let input_dir = Path::new("post");
    let output_dir = Path::new("public/posts");

    if !input_dir.exists() {
        bail!("缺少输入目录: {}", input_dir.display());
    }

    if clean {
        clean_output_dir()?;
    }
    fs::create_dir_all(output_dir)
        .with_context(|| format!("无法创建目录: {}", output_dir.display()))?;

    let site = load_site_config();
    copy_theme_assets(site.theme.as_str())
        .with_context(|| format!("复制主题资源失败: {}", site.theme))?;

    let post_dirs = collect_post_dirs(input_dir)?;
    if post_dirs.is_empty() {
        bail!("未找到有效文章目录（需要 post/<id>/index.typ 与 meta.toml）");
    }

    let mut entries: Vec<(PathBuf, PostMeta)> = Vec::new();
    for dir in &post_dirs {
        let meta = post_meta_from_post_dir(dir)?;
        entries.push((dir.clone(), meta));
    }

    for (dir, meta) in &entries {
        if meta.draft {
            if verbose {
                println!("跳过草稿: {}", dir.display());
            }
            continue;
        }
        let input = dir.join("index.typ");
        let id = meta.id.as_str();
        let out_dir = output_dir.join(id);
        let output = out_dir.join("index.html");

        if verbose {
            println!("编译: {} -> {}", input.display(), output.display());
        }

        let date_str = meta
            .date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        run_typst_compile(&input, &output, &meta.title, &date_str)?;
        copy_post_assets(dir, &out_dir)?;
        let raw = fs::read_to_string(&output)
            .with_context(|| format!("无法读取 {}", output.display()))?;
        let patched = inject_theme_post_html(&raw, &site, meta);
        fs::write(&output, patched)
            .with_context(|| format!("无法写入 {}", output.display()))?;
    }

    let mut index_metas: Vec<PostMeta> = entries
        .iter()
        .filter(|(_, m)| !m.draft)
        .map(|(_, m)| m.clone())
        .collect();
    sort_posts_desc(&mut index_metas);

    let index_path = Path::new("public/index.html");
    write_index_html(index_path, &site, &index_metas)?;

    println!(
        "完成: 已生成 {} 与 {}",
        index_path.display(),
        output_dir.display()
    );

    validate_generated_site().context("构建产物校验失败")?;
    Ok(())
}

/// 校验 `public/index.html` 与 `public/posts/<id>/index.html` 与非草稿文章一致且内容像 HTML。
pub fn validate_generated_site() -> Result<()> {
    validate_generated_site_paths(Path::new("post"), Path::new("public"))
}

fn validate_generated_site_paths(post_root: &Path, public_root: &Path) -> Result<()> {
    let public_posts = public_root.join("posts");
    let public_index = public_root.join("index.html");

    let dirs = collect_post_dirs(post_root)?;
    let mut expected: Vec<String> = Vec::new();
    for dir in &dirs {
        let meta = post_meta_from_post_dir(dir)?;
        if !meta.draft {
            expected.push(meta.id.clone());
        }
    }
    expected.sort();

    let index_raw = fs::read_to_string(&public_index)
        .with_context(|| format!("缺少或无法读取 {}", public_index.display()))?;
    if !html_looks_valid(&index_raw) {
        bail!("{} 内容异常（缺少 HTML 标记）", public_index.display());
    }

    for id in &expected {
        let html_path = public_posts.join(id).join("index.html");
        let raw = fs::read_to_string(&html_path)
            .with_context(|| format!("缺少或无法读取期望输出: {}", html_path.display()))?;
        if !html_looks_valid(&raw) {
            bail!("{} 内容异常（缺少 HTML 标记）", html_path.display());
        }
    }

    let mut actual: Vec<String> = Vec::new();
    if public_posts.is_dir() {
        for entry in fs::read_dir(&public_posts)
            .with_context(|| format!("无法读取 {}", public_posts.display()))?
        {
            let entry = entry?;
            if !entry
                .file_type()
                .with_context(|| format!("无法读取文件类型: {}", entry.path().display()))?
                .is_dir()
            {
                continue;
            }
            let name = entry.file_name();
            let Some(id) = name.to_str() else {
                bail!("文章目录名须为 UTF-8");
            };
            if public_posts.join(id).join("index.html").is_file() {
                actual.push(id.to_string());
            }
        }
    }
    actual.sort();

    if expected != actual {
        bail!("public/posts 下产物与应发布文章不一致: 期望 {expected:?} 实际 {actual:?}");
    }

    Ok(())
}

fn html_looks_valid(s: &str) -> bool {
    let head = s.get(..4096.min(s.len())).unwrap_or(s).to_ascii_lowercase();
    head.contains("<!doctype html") || head.contains("<html")
}

pub fn clean_output_dir() -> Result<()> {
    let output_dir = Path::new("public/");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .with_context(|| format!("无法清理目录: {}", output_dir.display()))?;
    }
    fs::create_dir_all(output_dir)
        .with_context(|| format!("无法创建目录: {}", output_dir.display()))?;
    println!("已清理: {}", output_dir.display());
    Ok(())
}

fn run_typst_compile(input: &Path, output: &Path, title: &str, date: &str) -> Result<()> {
    let root = std::env::current_dir().context("无法获取当前工作目录")?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {}", parent.display()))?;
    }
    let status = Command::new("typst")
        .current_dir(&root)
        .arg("compile")
        .arg("--root")
        .arg(&root)
        .arg("--input")
        .arg(format!("title={title}"))
        .arg("--input")
        .arg(format!("date={date}"))
        .arg("--features")
        .arg("html")
        .arg("--format")
        .arg("html")
        .arg(input)
        .arg(output)
        .status()
        .with_context(|| format!("执行 typst 失败: {}", input.display()))?;

    if !status.success() {
        bail!("typst 编译失败: {}", input.display());
    }
    Ok(())
}

/// 将 post/<id>/ 下除 index.typ、meta.toml 外的文件与子目录复制到输出目录。
fn copy_post_assets(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(to).with_context(|| format!("无法创建目录: {}", to.display()))?;
    for entry in fs::read_dir(from).with_context(|| format!("无法读取目录: {}", from.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if name == "index.typ" || name == "meta.toml" {
            continue;
        }
        let dest = to.join(&name);
        if path.is_dir() {
            copy_dir_all(&path, &dest)?;
        } else if path.is_file() {
            fs::copy(&path, &dest).with_context(|| {
                format!("复制资源失败: {} -> {}", path.display(), dest.display())
            })?;
        }
    }
    Ok(())
}

/// 将 `themes/<theme>/assets/` 复制到 `public/assets/themes/<theme>/`。
fn copy_theme_assets(theme: &str) -> Result<()> {
    let src = Path::new("themes").join(theme).join("assets");
    let dst = Path::new("public").join("assets").join("themes").join(theme);
    if !src.is_dir() {
        bail!(
            "主题资源目录不存在: {}（请将 themes/{}/assets 置于仓库根，或运行 typlog init）",
            src.display(),
            theme
        );
    }
    copy_dir_all(&src, &dst)?;
    Ok(())
}

fn copy_dir_all(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to).with_context(|| format!("无法创建目录: {}", to.display()))?;
    for entry in fs::read_dir(from).with_context(|| format!("无法读取目录: {}", from.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let dest = to.join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &dest)?;
        } else if path.is_file() {
            fs::copy(&path, &dest).with_context(|| {
                format!("复制资源失败: {} -> {}", path.display(), dest.display())
            })?;
        }
    }
    Ok(())
}

/// 列出 `post/<id>/` 目录：必须同时存在 `index.typ` 与 `meta.toml`。
fn collect_post_dirs(post_root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !post_root.is_dir() {
        return Ok(out);
    }
    for entry in
        fs::read_dir(post_root).with_context(|| format!("无法读取目录: {}", post_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let index = path.join("index.typ");
        let meta = path.join("meta.toml");
        if !index.is_file() {
            continue;
        }
        if !meta.is_file() {
            bail!(
                "缺少 meta.toml: {}（每篇文章目录需要 meta.toml 供博客元数据使用）",
                path.display()
            );
        }
        out.push(path);
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_passes_when_posts_match_public() {
        let dir = std::env::temp_dir().join(format!("typlog-validate-ok-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("post/a")).unwrap();
        fs::write(
            dir.join("post/a/meta.toml"),
            r#"title = "A"
date = "2026-01-01"
draft = false
"#,
        )
        .unwrap();
        fs::write(dir.join("post/a/index.typ"), "#set text[]\nok").unwrap();
        fs::create_dir_all(dir.join("public/posts/a")).unwrap();
        fs::write(
            dir.join("public/index.html"),
            "<!DOCTYPE html><html><body></body></html>",
        )
        .unwrap();
        fs::write(
            dir.join("public/posts/a/index.html"),
            "<!DOCTYPE html><html><body></body></html>",
        )
        .unwrap();
        validate_generated_site_paths(&dir.join("post"), &dir.join("public")).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_fails_when_extra_public_dir() {
        let dir =
            std::env::temp_dir().join(format!("typlog-validate-extra-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("post/a")).unwrap();
        fs::write(
            dir.join("post/a/meta.toml"),
            r#"title = "A"
date = "2026-01-01"
draft = false
"#,
        )
        .unwrap();
        fs::write(dir.join("post/a/index.typ"), "#x").unwrap();
        fs::create_dir_all(dir.join("public/posts/a")).unwrap();
        fs::create_dir_all(dir.join("public/posts/orphan")).unwrap();
        fs::write(
            dir.join("public/posts/orphan/index.html"),
            "<!DOCTYPE html><html></html>",
        )
        .unwrap();
        fs::write(
            dir.join("public/index.html"),
            "<!DOCTYPE html><html></html>",
        )
        .unwrap();
        fs::write(
            dir.join("public/posts/a/index.html"),
            "<!DOCTYPE html><html></html>",
        )
        .unwrap();
        assert!(validate_generated_site_paths(&dir.join("post"), &dir.join("public")).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
