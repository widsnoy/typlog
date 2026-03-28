use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config::read_site_title;
use crate::html::write_index_html;
use crate::meta::{PostMeta, post_meta_from_file, sort_posts_desc};

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

    let posts = collect_post_index_files(input_dir)?;
    if posts.is_empty() {
        bail!("未找到任何 post/<slug>/index.typ，请按该结构添加文章");
    }

    for input in &posts {
        let slug = input
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .context("无法从路径解析 slug")?;
        let out_dir = output_dir.join(slug);
        let output = out_dir.join("index.html");

        if verbose {
            println!("编译: {} -> {}", input.display(), output.display());
        }

        run_typst_compile(input, &output)?;

        let src_dir = input.parent().context("文章目录")?;
        copy_post_assets(src_dir, &out_dir)?;
    }

    let mut metas: Vec<PostMeta> = posts
        .iter()
        .map(|p| post_meta_from_file(p))
        .collect::<Result<_>>()?;
    sort_posts_desc(&mut metas);

    let site_title = read_site_title();
    let index_path = Path::new("public/index.html");
    write_index_html(index_path, &site_title, &metas)?;

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

fn run_typst_compile(input: &Path, output: &Path) -> Result<()> {
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

/// 将 post/<slug>/ 下除 index.typ 外的文件与子目录复制到输出目录（图片等资源）。
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
        if name == "index.typ" {
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

fn collect_post_index_files(post_root: &Path) -> Result<Vec<PathBuf>> {
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
        if index.is_file() {
            out.push(index);
        }
    }
    out.sort();
    Ok(out)
}
