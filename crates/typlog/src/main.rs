use anyhow::Result;
use clap::{Parser, Subcommand};
use typlog_core::{clean_output_dir, generate, init_workspace, new_post, validate_generated_site};

mod server;

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
        dir: std::path::PathBuf,
    },
    /// 在 post/<id>/ 下创建 meta.toml 与 index.typ（id 为 kebab-case 目录名）
    New {
        /// 文章 id（目录名，用于 URL 路径段）
        id: String,
    },
    /// 编译 post/<id>/index.typ → public/posts/<id>/index.html（元数据来自 meta.toml）
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
    /// 校验 public/ 与非草稿文章数一致（无需重新编译）
    Validate,
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
        Commands::New { id } => new_post(&id),
        Commands::Generate { clean, verbose } => generate(clean, verbose),
        Commands::Clean => clean_output_dir(),
        Commands::Validate => validate_generated_site(),
        Commands::Server { port } => server::serve_public(port),
    }
}
