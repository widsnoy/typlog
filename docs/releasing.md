# 发布版本

1. 确保 `main`（或发布分支）上代码与 `cargo test`、`cargo clippy` 已通过。
2. 打 **annotated 或 lightweight 标签**（建议 [SemVer](https://semver.org/lang/zh-CN/)）：
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```
3. GitHub Actions 工作流 **Release**（见 `.github/workflows/release.yml`）会：
   - 构建 Linux / Windows 二进制并打包；
   - 生成 **自上一 `v*` 标签以来** 的提交列表，写入 Release 说明；
   - 创建 **GitHub Release** 并上传 Assets。

首仓库次打 `v*` 标签时，说明中会列出当前可达的提交历史（无更早 `v*` 标签时）。
