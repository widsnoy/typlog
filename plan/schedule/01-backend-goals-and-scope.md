# 01 后端目标与范围

**实现状态**：目标与失败策略已满足；**Typst 版本锁定**（见 04）尚未写入仓库；CI 未接。

## 目标

在 **不依赖首页、样式、复杂导航** 的前提下，建立可重复的构建后端，使任意文章源文件可被稳定编译为 HTML 产物。

## 后端交付物（必须）

- 明确的 **Typst 版本** 与启用 HTML 导出所需的 **feature**（当前实现为 `typst compile --features html --format html`；**精确版本号待写入仓库**，见 04）。
- **`post/<文章 id>/` → `public/posts/<文章 id>/index.html`** 的映射规则文档化（见 [02](02-backend-directory-and-metadata.md)）。
- **构建入口**（脚本或任务）：单命令全量构建；**不依赖 npm/Node**；对标 Hexo 式子命令与配置见 [07-hexo-like-cli-and-config.md](07-hexo-like-cli-and-config.md)。可选 watch 仅作本地开发便利，非首版必须。
- **失败策略**：任一篇编译失败 → 进程非 0 退出；不在失败时留下半套“看似成功”的产物（或文档说明清理行为）。

## 明确排除（属前端阶段）

- 首页、归档页、标签页的 **完整版式与交互**（当前仅有 **极简** `public/index.html` 列表，见 [05](05-frontend-shell-and-routing.md)）。
- 全局 CSS 主题、响应式细节、字体栈（构建阶段仅需保证 HTML 可打开即可）。
- SEO 的 `meta`/`og`、RSS、sitemap（可在 06 或后续迭代）。

## 里程碑

- **M1**：单篇 `post/<id>/index.typ`（及 `meta.toml`）→ `public/posts/<id>/index.html` 可重复成功（**已实现**）。
- **M2**：目录内多篇批量成功，且 `typlog generate` / `clean` 行为可预期（**已实现**）。
- **M3**：CI 仅执行构建与校验，绿即表示后端阶段可用（**未实现**，见 04）。

## 风险与缓冲

- HTML 导出仍在演进：为 Typst 小版本升级预留 **0.5～1 天** 回归（抽样 3 篇文章 + 1 篇含公式/代码）。
