use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use chrono::Local;
use clap::{Parser, Subcommand};
use tiny_http::{Header, Response, Server, StatusCode};

#[derive(Parser, Debug)]
#[command(name = "typlog", version, about = "Typst blog tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new typlog workspace
    Init {
        /// Target directory, defaults to current directory
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    /// Create a new post under post/<slug>.typ
    New {
        /// Post slug, use kebab-case
        slug: String,
    },
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
    /// Preview the generated static site under public/
    Server {
        /// HTTP listen port
        #[arg(long, default_value_t = 4000)]
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { dir } => init_workspace(&dir),
        Commands::New { slug } => new_post(&slug),
        Commands::Generate { clean, verbose } => generate(clean, verbose),
        Commands::Clean => clean_output_dir(),
        Commands::Server { port } => serve_public(port),
    }
}

fn init_workspace(dir: &Path) -> Result<()> {
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

    let template_path = dir.join("templates/post.typ");
    if !template_path.exists() {
        let template = r#"#let title = "文章标题"
#let date = "2026-03-28"

= #title

日期：#date

在这里开始写正文。
"#;
        fs::write(&template_path, template)
            .with_context(|| format!("无法写入文件: {}", template_path.display()))?;
    }

    println!("初始化完成: {}", dir.display());
    Ok(())
}

fn new_post(slug: &str) -> Result<()> {
    validate_slug(slug)?;
    let post_dir = Path::new("post");
    fs::create_dir_all(post_dir)
        .with_context(|| format!("无法创建目录: {}", post_dir.display()))?;

    let post_path = post_dir.join(format!("{slug}.typ"));
    if post_path.exists() {
        bail!("文章已存在: {}", post_path.display());
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let content = format!(
        "#let title = \"{slug}\"\n#let date = \"{today}\"\n\n= #title\n\n日期：#date\n\n在这里开始写正文。\n"
    );
    fs::write(&post_path, content)
        .with_context(|| format!("无法写入文件: {}", post_path.display()))?;
    println!("已创建: {}", post_path.display());
    Ok(())
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

fn serve_public(port: u16) -> Result<()> {
    let root = Path::new("public");
    if !root.exists() {
        bail!("缺少 public/ 目录，请先执行 generate");
    }

    let address = format!("127.0.0.1:{port}");
    let server = Server::http(&address).map_err(|e| anyhow!("无法监听地址 {address}: {e}"))?;
    println!("本地预览已启动: http://{address}");

    for request in server.incoming_requests() {
        let request_url = request.url().to_string();
        let url_path = request_url.split('?').next().unwrap_or("/");
        let rel = url_path.trim_start_matches('/');
        let mut fs_path = if rel.is_empty() {
            root.join("index.html")
        } else {
            root.join(rel)
        };

        if fs_path.is_dir() {
            fs_path = fs_path.join("index.html");
        }
        if !fs_path.exists() && rel.is_empty() {
            fs_path = root.join("posts");
        }
        if fs_path.is_dir() {
            fs_path = fs_path.join("index.html");
        }

        let response = match fs::File::open(&fs_path) {
            Ok(mut file) => {
                let mut body = Vec::new();
                file.read_to_end(&mut body)?;
                let mut resp = Response::from_data(body);
                if let Ok(header) = Header::from_bytes(
                    b"Content-Type".as_slice(),
                    guess_content_type(&fs_path).as_bytes(),
                ) {
                    resp = resp.with_header(header);
                }
                resp
            }
            Err(_) => Response::from_string("Not Found").with_status_code(StatusCode(404)),
        };
        let _ = request.respond(response);
    }
    Ok(())
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    }
}

fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        bail!("slug 不能为空");
    }
    let is_valid = slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !is_valid {
        bail!("slug 仅允许小写字母、数字、短横线");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{guess_content_type, validate_slug};

    #[test]
    fn slug_should_pass_when_kebab_case() {
        assert!(validate_slug("hello-2026").is_ok());
    }

    #[test]
    fn slug_should_fail_when_has_uppercase() {
        assert!(validate_slug("Hello").is_err());
    }

    #[test]
    fn content_type_should_detect_html() {
        assert_eq!(
            guess_content_type(Path::new("a/b/c.html")),
            "text/html; charset=utf-8"
        );
    }
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
