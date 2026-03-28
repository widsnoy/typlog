# 工具链版本

开发与 CI 请使用一致的 **Typst** 版本，避免 HTML 导出行为漂移。

| 工具 | 版本 | 说明 |
| --- | --- | --- |
| **Rust** | stable | `cargo build` / `cargo test` / `cargo clippy` |
| **Typst** | **0.14.2** | `typlog generate` 调用 `typst compile`；需支持 `--features html` |

安装 Typst 请参考 [官方 Releases](https://github.com/typst/typst/releases)（与本表版本对齐）。CI 从同一版本下载 Linux 二进制。

**发版**：推送 `v*` 标签触发 Release 工作流，见 [releasing.md](releasing.md)。
