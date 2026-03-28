use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config::load_site_config;
use crate::html::write_index_html;
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
    }

    let mut index_metas: Vec<PostMeta> = entries
        .iter()
        .filter(|(_, m)| !m.draft)
        .map(|(_, m)| m.clone())
        .collect();
    sort_posts_desc(&mut index_metas);

    let site_title = load_site_config().title;
    let index_path = Path::new("public/index.html");
    write_index_html(index_path, &site_title, &index_metas)?;

    println!(
        "完成: 已生成 {} 与 {}",
        index_path.display(),
        output_dir.display()
    );
    Ok(())
}

pub fn clean_output_dir() -> Result<()> {
    let output_dir = Path::new("public/posts");
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
