# 07 工具链：对标 Hexo 的体验（核心构建不依赖 npm）

**实现状态**：Rust 二进制 **`typlog`**（`crates/typlog`）已实现下表子命令；`config.toml` 使用 **`serde` 反序列化 `SiteConfig`**（`title`、`base_url`、`language`）；`init` 写入默认配置为 **`toml::to_string(&SiteConfig::default())`**。**已完成**（与 Hexo 对齐的交互；细节以代码为准）。

## 约束（硬）：后端 / 核心构建

以下针对 **`typlog generate`**、仓库 **默认端到端构建** 与 **CI**（见 [04](04-backend-validation-and-ci.md)）：

- **不得**依赖 **Node.js / npm / pnpm / yarn** 作为必经路径（无强制 `package.json`、`node_modules` 才能编译文章与写 `public/`）。
- **不推荐**用 npm 脚本封装再调 Typst 作为**唯一**入口；默认入口仍是 **`typlog`**（Rust）。

## 前端（允许 npm 等）

- **主题、全局样式、组件化资源** 可使用 **npm / Vite / 任意前端工具** 开发与打包，只要：
  - **与核心构建解耦**：CI 仍只跑 `cargo …`、`typlog generate`（及固定 Typst）；不强制贡献者安装 Node 才能通过后端构建。
  - **产物落点明确**：例如打包输出写入 `themes/<name>/assets/` 或约定目录，再由 `generate` 复制进 `public/`，或文档说明「先 `npm run build` 主题再 `typlog generate`」的本地流程。
- 主题与生成器如何协作（meta/正文嵌入）：见 [08](08-frontend-architecture.md)；其余见 [05](05-frontend-shell-and-routing.md)、[06](06-frontend-quality-and-release.md)。

## 目标体验（与 Hexo 对齐的交互）

| Hexo 概念 | 本项目目标 | 说明 |
| --- | --- | --- |
| `hexo init` | `typlog init [目录]` | 脚手架：目录、`config.toml` 样例、`post/`、`templates/`、`public/` 占位 |
| `hexo new <title>` | `typlog new <id>` | 按模板生成 `post/<id>/`（`meta.toml` + `index.typ`） |
| `hexo generate` | `typlog generate` | 全量编译 + 写 `public/index.html` 列表 |
| `hexo clean` | `typlog clean` | 清理 `public/posts/`（保留根目录其它文件） |
| `hexo server` | `typlog server --port <端口>` | 本地静态预览 `public/` |
| （校验） | `typlog validate` | 检查 `public/` 与非草稿文章一致（`generate` 已内含） |
| `_config.yml` | `config.toml` | 站点 `title`、`base_url`、`language`（与文章 `meta.toml` 分离） |

## 实现语言（计划定稿）

- **首选：Rust** 实现 `typlog` CLI（单仓库内 `cargo` 工程，发布为单一静态二进制）——**已采用**。
- 原型阶段可例外：Just / Makefile 过渡——**非必须**；当前无强制 Makefile。

**不计划采用** Python 作为 `typlog` 的默认实现。**核心构建主路径** 不是 Node/npm（见上文「约束（硬）」）。

## 实现路径（与语言对应）

1. **目标形态**：**Rust 二进制** `typlog`；仓库根 `cargo build -p typlog` 产出可执行文件。
2. **核心入口**：不推荐用 **仅** npm 封装再调 Typst 作为仓库唯一构建方式；若主题子项目自带 `package.json`，应为**可选**前端子树，不替代 `typlog generate`。

## 与 schedule 其它文件的对应关系

- **generate / clean** 与 [03-backend-build-script.md](03-backend-build-script.md)、[04-backend-validation-and-ci.md](04-backend-validation-and-ci.md) 一致：CI 目标为只跑 `typlog generate` 等价命令（CI 待 04）。
- **init / new**：见 [02-backend-directory-and-metadata.md](02-backend-directory-and-metadata.md)；标题与日期以 `meta.toml` 为准，由 `generate` 经 `typst --input` 注入。
- **server**：本地预览；与 [05-frontend-shell-and-routing.md](05-frontend-shell-and-routing.md) 联调。

## 验收清单

- [x] **核心构建**文档与 CI 路径不出现 npm 脚本作为**必经**步骤；前端子项目可自带 npm，但不作为默认后端依赖。
- [ ] `typlog generate` 在 CI 与本地一致（CI 待 04）。
- [x] `typlog new` 生成文件符合 02 中元数据契约。
- [x] `typlog server` 能打开首页与文章（本地验证）；部署子路径待 05。

## 时间估算（参考）

- Rust CLI 完整子命令：**已在迭代中完成初版**；后续为校验与体验打磨。
