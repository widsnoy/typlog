# 07 工具链：对标 Hexo 的体验（不依赖 npm）

**实现状态**：Rust 二进制 **`typlog`**（`crates/typlog`）已实现下表子命令；`config.toml` 使用 **`serde` 反序列化 `SiteConfig`**（`title`、`base_url`、`language`）；`init` 写入默认配置为 **`toml::to_string(&SiteConfig::default())`**。**已完成**（与 Hexo 对齐的交互；细节以代码为准）。

## 约束（硬）

- **构建与站点生成** 不得依赖 **Node.js / npm / pnpm / yarn**（无 `package.json` 作为必经路径、无 `node_modules`）。
- 可选：未来若为主题开发单独提供「前端资源打包」，也须与 **正文构建** 解耦，且不作为默认路径。

## 目标体验（与 Hexo 对齐的交互）

| Hexo 概念 | 本项目目标 | 说明 |
| --- | --- | --- |
| `hexo init` | `typlog init [目录]` | 脚手架：目录、`config.toml` 样例、`post/`、`templates/`、`public/` 占位 |
| `hexo new <title>` | `typlog new <id>` | 按模板生成 `post/<id>/`（`meta.toml` + `index.typ`） |
| `hexo generate` | `typlog generate` | 全量编译 + 写 `public/index.html` 列表 |
| `hexo clean` | `typlog clean` | 清理 `public/posts/`（保留根目录其它文件） |
| `hexo server` | `typlog server --port <端口>` | 本地静态预览 `public/` |
| `_config.yml` | `config.toml` | 站点 `title`、`base_url`、`language`（与文章 `meta.toml` 分离） |

## 实现语言（计划定稿）

- **首选：Rust** 实现 `typlog` CLI（单仓库内 `cargo` 工程，发布为单一静态二进制）——**已采用**。
- 原型阶段可例外：Just / Makefile 过渡——**非必须**；当前无强制 Makefile。

**不计划采用** Python 作为默认实现；**不采用** Node/npm 作为构建主路径（见上文硬约束）。

## 实现路径（与语言对应）

1. **目标形态**：**Rust 二进制** `typlog`；仓库根 `cargo build -p typlog` 产出可执行文件。
2. **不推荐**：用 npm 封装一层再调 Typst；长期维护 Bash + PowerShell 双份入口。

## 与 schedule 其它文件的对应关系

- **generate / clean** 与 [03-backend-build-script.md](03-backend-build-script.md)、[04-backend-validation-and-ci.md](04-backend-validation-and-ci.md) 一致：CI 目标为只跑 `typlog generate` 等价命令（CI 待 04）。
- **init / new**：见 [02-backend-directory-and-metadata.md](02-backend-directory-and-metadata.md)；标题与日期以 `meta.toml` 为准，由 `generate` 经 `typst --input` 注入。
- **server**：本地预览；与 [05-frontend-shell-and-routing.md](05-frontend-shell-and-routing.md) 联调。

## 验收清单

- [x] 文档路径不出现 npm 脚本作为必经构建步骤。
- [ ] `typlog generate` 在 CI 与本地一致（CI 待 04）。
- [x] `typlog new` 生成文件符合 02 中元数据契约。
- [x] `typlog server` 能打开首页与文章（本地验证）；部署子路径待 05。

## 时间估算（参考）

- Rust CLI 完整子命令：**已在迭代中完成初版**；后续为校验与体验打磨。
