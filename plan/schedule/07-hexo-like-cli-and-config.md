# 07 工具链：对标 Hexo 的体验（不依赖 npm）

## 约束（硬）

- **构建与站点生成** 不得依赖 **Node.js / npm / pnpm / yarn**（无 `package.json` 作为必经路径、无 `node_modules`）。
- 可选：未来若为主题开发单独提供「前端资源打包」，也须与 **正文构建** 解耦，且不作为默认路径。

## 目标体验（与 Hexo 对齐的交互）

| Hexo 概念 | 本项目目标 | 说明 |
| --- | --- | --- |
| `hexo init` | `typlog init [目录]` | 脚手架：目录、`config` 样例、`post/`、`templates/`、`public/` 占位 |
| `hexo new <title>` | `typlog new <id>` | 按模板生成 `post/<id>/`（`meta.toml` + `index.typ`） |
| `hexo generate` | `typlog generate`（或 `build`） | 全量编译 + 生成列表等产物 |
| `hexo clean` | `typlog clean` | 清理约定输出目录（如 `public/posts/`、生成的 index） |
| `hexo server` | `typlog server` | 本地静态预览 `public/`，默认端口可配置 |
| `_config.yml` | `config.toml` 或 `site.yaml` | 站点标题、base URL、作者、语言等（与 Typst 正文元数据分离） |

命令名 **`typlog`** 为占位；实现时可改为项目实际二进制名或 `scripts/typlog` 入口。

## 实现语言（计划定稿）

- **首选：Rust** 实现 `typlog` CLI（单仓库内 `cargo` 工程，发布为单一静态二进制）。
  - **理由**：与 Typst 工具链文化一致；`clap` + `serde` 配置解析成熟；无额外运行时；跨平台交叉编译与 GitHub Release 分发常见；后续若要与 Typst 生态库复用概念，路径清晰。
- **原型阶段可例外**：在 CLI 骨架未就绪前，可用 **Justfile** 或 **Makefile** 临时映射 `generate`/`clean`（文档标明为过渡，最终由 Rust 二进制接管）。

**不计划采用** Python 作为默认实现（避免 CI/用户机 Python 版本漂移）；**不采用** Node/npm 作为构建主路径（见上文硬约束）。

## 实现路径（与语言对应）

1. **过渡**：Just / Makefile 仅用于早期验证 `typst compile` 批量与目录约定。
2. **目标形态**：**Rust 二进制** `typlog`，子命令见上表；仓库根 `cargo build --release` 产出可执行文件，CI 缓存 `target/` 或直接用 `cargo install --path .`。
3. **不推荐**：用 npm 封装一层再调 Typst；**不推荐**：长期维护 Bash + PowerShell 双份入口（除非生成器从单一源生成）。

## 与 schedule 其它文件的对应关系

- **generate / clean** 与 [03-backend-build-script.md](03-backend-build-script.md)、[04-backend-validation-and-ci.md](04-backend-validation-and-ci.md) 一致：CI 只跑与 `generate` 等价的非交互命令。
- **init / new**：主要影响 [02-backend-directory-and-metadata.md](02-backend-directory-and-metadata.md) 的模板与契约；默认 `templates/post.typ` 为**自包含**版式；标题与日期以 `meta.toml` 为准，由 generate 经 `--input` 注入。
- **server**：本地预览；与 [05-frontend-shell-and-routing.md](05-frontend-shell-and-routing.md) 联调（确保 `public/` 可完整浏览）。

## 验收清单

- [ ] 文档中从克隆到「生成可浏览站点」的路径 **不出现** npm 脚本。
- [ ] `typlog generate`（或选定等价命令）在 CI 与本地一致。
- [ ] `typlog new` 生成文件符合 02 中元数据契约。
- [ ] `typlog server` 能打开首页与至少一篇文章（在 05 完成后验收）。

## 时间估算（参考）

- 配置格式 + generate/clean（Just/Makefile 过渡）：**0.5～1 天**
- Rust CLI 完整子命令 + 跨平台：**约 3～7 人日**（视子命令范围与错误处理深度）
