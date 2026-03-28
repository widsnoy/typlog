use anyhow::Result;
use clap::{Parser, Subcommand};
use typlog_core::{clean_output_dir, generate, init_workspace, new_post, validate_generated_site};

mod server;

#[derive(Parser, Debug)]
#[command(name = "typlog", version, about = "Typst 静态博客：文章、构建与本地预览")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 在目标目录创建 posts/、themes/、templates/ 等（默认当前目录）
    Init {
        /// 目标目录
        #[arg(default_value = ".")]
        dir: std::path::PathBuf,
    },
    /// 在 posts/<id>/ 下创建 meta.toml 与 index.typ（id 为 kebab-case 目录名）
    New {
        /// 文章 id（目录名，用作 URL 路径段）
        id: String,
    },
    /// 将 posts/<id>/index.typ 编译为 public/posts/<id>/index.html，并生成首页（元数据来自 meta.toml）
    Generate {
        /// 生成前先删除整个 public/ 目录
        #[arg(long)]
        clean: bool,
        /// 打印每篇文章的 typst 编译命令
        #[arg(long)]
        verbose: bool,
    },
    /// 删除整个 public/ 目录（仅清理，不编译）
    Clean,
    /// 校验 public/ 与非草稿文章是否一致（无需重新编译）
    Validate,
    /// 在本地 HTTP 服务中预览 public/ 静态站
    Server {
        /// 监听端口
        #[arg(long, default_value_t = 4000)]
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { dir } => init_workspace(&dir),
        Commands::New { id } => new_post(&id),
        Commands::Generate { clean, verbose } => generate(clean, verbose),
        Commands::Clean => clean_output_dir(),
        Commands::Validate => validate_generated_site(),
        Commands::Server { port } => server::serve_public(port),
    }
}
