//! Typlog 核心逻辑：文章扫描、Typst 构建、索引 HTML、脚手架（无 CLI / HTTP）。

pub mod build;
pub mod config;
pub mod html;
pub mod init;
pub mod meta;
pub mod post;
pub mod scaffold;

pub use build::{clean_output_dir, generate};
pub use config::{
    SiteConfig, default_site_config_toml, load_site_config, load_site_config_from_path,
};
pub use init::init_workspace;
pub use meta::{PostMeta, post_meta_from_post_dir, sort_posts_desc};
pub use post::{new_post, validate_post_id};
