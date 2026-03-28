use std::cmp::Ordering;
use std::fmt::Write;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{Local, NaiveDate};
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
    /// Create a new post at post/<slug>/index.typ
    New {
        /// Post slug, use kebab-case
        slug: String,
    },
    /// Compile post/<slug>/index.typ into public/posts/<slug>/index.html
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

fn new_post(slug: &str) -> Result<()> {
    validate_slug(slug)?;
    let post_root = Path::new("post");
    fs::create_dir_all(post_root)
        .with_context(|| format!("无法创建目录: {}", post_root.display()))?;

    let dir = post_root.join(slug);
    if dir.exists() {
        bail!("文章目录已存在: {}", dir.display());
    }
    fs::create_dir_all(&dir).with_context(|| format!("无法创建目录: {}", dir.display()))?;

    let post_path = dir.join("index.typ");
    let today = Local::now().format("%Y-%m-%d").to_string();
    let content = format!(
        r#"#import "/templates/article.typ": article

#article("{slug}", "{today}")[
在这里开始写正文。
]
"#,
        slug = slug,
        today = today,
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

struct PostMeta {
    slug: String,
    title: String,
    date: Option<NaiveDate>,
}

fn post_meta_from_file(path: &Path) -> Result<PostMeta> {
    let slug = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .context("无法从路径解析 slug")?
        .to_string();
    let content =
        fs::read_to_string(path).with_context(|| format!("无法读取 {}", path.display()))?;
    let (title, date) = if let Some((t, d)) = parse_article_call(&content) {
        let date = NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok();
        (t, date)
    } else {
        let title = parse_let_value(&content, "title").unwrap_or_else(|| slug.clone());
        let date = parse_let_value(&content, "date")
            .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
        (title, date)
    };
    Ok(PostMeta { slug, title, date })
}

/// 解析 `#article("标题", "日期")` 形式（均为位置参数与引号字符串）。
fn parse_article_call(content: &str) -> Option<(String, String)> {
    let idx = content.find("#article(")?;
    let mut rest = &content[idx + "#article(".len()..];
    let title = parse_leading_quoted(&mut rest)?;
    rest = rest.trim_start();
    rest = rest.strip_prefix(',')?.trim_start();
    let date = parse_leading_quoted(&mut rest)?;
    Some((title, date))
}

fn parse_leading_quoted(rest: &mut &str) -> Option<String> {
    let s = rest.trim_start();
    let s = s.strip_prefix('"')?;
    let end = s.find('"')?;
    let val = s[..end].to_string();
    *rest = &s[end + 1..];
    Some(val)
}

fn parse_let_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("#let {key} = \"");
    for line in content.lines().take(128) {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        return Some(rest[..end].to_string());
    }
    None
}

fn sort_posts_desc(posts: &mut [PostMeta]) {
    posts.sort_by(|a, b| match (&a.date, &b.date) {
        (Some(da), Some(db)) => db.cmp(da),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.slug.cmp(&b.slug),
    });
}

fn read_site_title() -> String {
    let path = Path::new("config.toml");
    let Ok(content) = fs::read_to_string(path) else {
        return "Typlog".to_string();
    };
    for line in content.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if !line.starts_with("title") {
            continue;
        }
        let Some(eq) = line.find('=') else {
            continue;
        };
        let val = line[eq + 1..].trim();
        if let Some(inner) = val.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            return inner.to_string();
        }
    }
    "Typlog".to_string()
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn write_index_html(out: &Path, site_title: &str, posts: &[PostMeta]) -> Result<()> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建目录: {}", parent.display()))?;
    }
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<title>");
    html.push_str(&html_escape(site_title));
    html.push_str("</title>\n</head>\n<body>\n");
    html.push_str("<h1>");
    html.push_str(&html_escape(site_title));
    html.push_str("</h1>\n<ul>\n");
    for p in posts {
        html.push_str("<li><a href=\"posts/");
        html.push_str(&p.slug);
        html.push_str("/index.html\">");
        html.push_str(&html_escape(&p.title));
        html.push_str("</a>");
        if let Some(d) = p.date {
            let _ = write!(&mut html, " <span>{}</span>", d.format("%Y-%m-%d"));
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ul>\n</body>\n</html>\n");
    fs::write(out, html).with_context(|| format!("无法写入 {}", out.display()))?;
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
        Some("webp") => "image/webp",
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

    use super::{
        guess_content_type, html_escape, parse_article_call, parse_let_value, validate_slug,
    };

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

    #[test]
    fn parse_let_should_read_title_and_date() {
        let s = r#"#let title = "Hello"
#let date = "2026-01-02"
"#;
        assert_eq!(parse_let_value(s, "title").as_deref(), Some("Hello"));
        assert_eq!(parse_let_value(s, "date").as_deref(), Some("2026-01-02"));
    }

    #[test]
    fn html_escape_should_escape_special_chars() {
        assert_eq!(html_escape("&<>"), "&amp;&lt;&gt;");
        assert_eq!(html_escape("\""), "&quot;");
    }

    #[test]
    fn parse_article_call_should_read_title_and_date() {
        let s = r#"#import "/templates/article.typ": article

#article("Hello", "2026-01-02")[
正文
]
"#;
        let (t, d) = parse_article_call(s).expect("article");
        assert_eq!(t, "Hello");
        assert_eq!(d, "2026-01-02");
    }
}
