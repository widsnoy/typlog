use anyhow::Result;
use clap::{Parser, Subcommand};
use typlog_core::{clean_output_dir, generate, init_workspace, new_post};

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
        Commands::Server { port } => server::serve_public(port),
    }
}
