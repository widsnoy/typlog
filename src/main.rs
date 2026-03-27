use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "typlog", version, about = "Typst blog tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compile all post/*.typ into public/posts/*.html
    Generate {
        /// Remove existing output directory before compiling
        #[arg(long)]
        clean: bool,
        /// Show each compile command
        #[arg(long)]
        verbose: bool,
    },
    /// Remove generated files under public/posts
    Clean,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate { clean, verbose } => generate(clean, verbose),
        Commands::Clean => clean_output_dir(),
    }
}

fn generate(clean: bool, verbose: bool) -> Result<()> {
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

    let posts = collect_typ_files(input_dir)?;
    if posts.is_empty() {
        bail!("未找到任何 .typ 文件，请先在 post/ 目录添加文章");
    }

    for input in posts {
        let slug = input
            .file_stem()
            .and_then(|s| s.to_str())
            .context("无法解析文件名为 slug")?;
        let output = output_dir.join(format!("{slug}.html"));

        if verbose {
            println!("编译: {} -> {}", input.display(), output.display());
        }

        run_typst_compile(&input, &output)?;
    }

    println!("完成: 已生成 HTML 到 {}", output_dir.display());
    Ok(())
}

fn run_typst_compile(input: &Path, output: &Path) -> Result<()> {
    let status = Command::new("typst")
        .arg("compile")
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

fn clean_output_dir() -> Result<()> {
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

fn collect_typ_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk_collect(dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_collect(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("无法读取目录: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_collect(&path, out)?;
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) == Some("typ") {
            out.push(path);
        }
    }
    Ok(())
}
