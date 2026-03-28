# 04 后端：校验与 CI

**实现状态**：**已完成（初版）**。见 `.github/workflows/ci.yml`、`docs/toolchain.md`；`typlog-core::validate_generated_site` 在 `generate` 末尾执行，并提供 **`typlog validate`**。推送 **`v*`** 标签时由 `.github/workflows/release.yml` 创建 **GitHub Release**（说明含提交列表、附构建产物），见 **`docs/releasing.md`**。

## 校验（构建后）

在 **不启动浏览器自动化** 的前提下，至少做：

- **计数 / 集合一致**：非草稿文章 id 集合与 `public/posts/<id>/`（且含 `index.html`）一致；**已实现**（`validate_generated_site`）。
- **重复文章 id**：同一文件系统下目录名唯一，**自然互斥**；若未来支持别名仍须防冲突。
- **HTML 粗检**：`public/index.html` 与各篇 `index.html` 片段内含 `<!DOCTYPE html` 或 `<html`（大小写不敏感）；**已实现**。

## CI 工作流（后端阶段）

- 触发：`push` / `pull_request` → `main` / `master`（见 `.github/workflows/ci.yml`）。
- 步骤：检出 → 安装 **Typst 0.14.2**（与 `docs/toolchain.md` 一致）→ `cargo clippy -D warnings` → `cargo test` → `typlog generate --clean --verbose` → **`typlog validate`**。
- **本阶段可不部署**。

## 版本锁定

- **Typst**：`docs/toolchain.md`；CI 环境变量 `TYPST_VERSION` 与之对齐。
- CI 与文档使用同一发行版资产（Linux `musl` 压缩包）。

## 验收清单

- [x] CI：`clippy` + `test` + `generate` + `validate`。
- [x] 步骤均在 workflow 与 `docs/toolchain.md` 可复现（**CI 无 npm 必经路径**；前端主题工具链不在此列）。

## 时间估算（参考）

- CI 初版：**0.5～1 天**
- 校验脚本：**0.5 天**
