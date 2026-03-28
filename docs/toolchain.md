# 工具链版本

开发与 CI 请使用一致的 **Typst** 版本，避免 HTML 导出行为漂移。

| 工具 | 版本 | 说明 |
| --- | --- | --- |
| **Rust** | stable | `cargo build` / `cargo test` / `cargo clippy` |
| **Typst** | **0.14.2** | `typlog generate` 调用 `typst compile`；需支持 `--features html` |

安装 Typst 请参考 [官方 Releases](https://github.com/typst/typst/releases)（与本表版本对齐）。CI 从同一版本下载 Linux 二进制。

**发版**：推送 `v*` 标签触发 Release 工作流，见 [releasing.md](releasing.md)。

## HTML 导出与数学公式

`typlog generate` 使用 Typst 的 **HTML 目标**（`--features html`、`--format html`）。该能力在官方文档中标注为**实验性**：行为可能变更，**不建议用于生产**。详见 [HTML – Typst Documentation](https://typst.app/docs/reference/html/)。进度与已知问题可跟踪 [typst#5512](https://github.com/typst/typst/issues/5512)。

**数学**：当前 HTML 导出对公式支持不完整。不设 workaround 时，Typst **0.14.2** 可能提示 `equation was ignored during HTML export`，HTML 中**不会出现**公式。

**默认做法（SVG）**：本仓库的 [`templates/post.typ`](templates/post.typ) 与 `typlog new` 使用的内置模板已对 `math.equation` 启用 [`html.frame`](https://typst.app/docs/reference/html/frame/)：在 `target() == "html"` 时用分页引擎将公式渲染为**内联 SVG**；PDF 等分页目标仍为默认数学排版。分支须写在外层 show rule（`html.frame` 内 `target()` 为分页）。旧文章可自 `templates/post.typ` 复制该段 `#show math.equation`。

**权衡**：公式以 SVG 呈现时，体积与可访问性策略与 MathML/前端 KaTeX 等方案不同。
