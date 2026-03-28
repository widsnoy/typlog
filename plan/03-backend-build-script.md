# 03 后端：构建脚本

**实现状态**：由 Rust crate `typlog-core::build` + CLI `typlog generate` 实现（**已完成**）。Typst 版本见 **`docs/toolchain.md`**，与 CI 一致（见 [04](04-backend-validation-and-ci.md)）。`generate` 末尾执行 **`validate_generated_site`**；亦可单独运行 **`typlog validate`**。

## 职责

- 扫描 `post/` 下 **一级子目录**（每篇 `post/<文章 id>/`），要求同时存在 `index.typ` 与 `meta.toml`（见 [02](02-backend-directory-and-metadata.md)）。
- 对每篇非草稿文章调用 `typst compile --root <仓库根> --input title=… --input date=… --features html --format html`，输出 `public/posts/<文章 id>/index.html`。
- 可选 **`--clean`**：先删除 `public/posts/` 再生成；**不**删除 `public/index.html`（由本次 `generate` 末尾重写）。
- 将站点标题等用于首页：读取 `config.toml`（`SiteConfig`），写 `public/index.html` 文章列表。

## 命令形态（逻辑要求）

- **禁止**以 `package.json` / `npm run` 作为默认构建路径；须与 [07-hexo-like-cli-and-config.md](07-hexo-like-cli-and-config.md) 一致（`typlog generate`）。
- 入口：仓库主入口为 **`cargo run -p typlog --` / 安装后的 `typlog`**；与 CI 应对齐同一命令（CI 待 04）。
- 参数：`--clean`、`--verbose`；已实现。

## 本地与 CI 一致性

- 目标：同一命令在本地与 CI 产生相同产物（CI 待 04）。
- Windows/Linux：路径由 Rust `Path` / `std::process` 处理，无硬编码分隔符脚本。

## 验收清单

- [x] 空 `post/`（或无有效文章目录）：构建 **失败** 并提示（与代码一致）。
- [x] 单篇成功：生成对应 `public/posts/<id>/index.html`。
- [x] 多篇成功：各篇均生成（**未**单独做「篇数 vs 文件数」校验，见 04）。
- [x] 故意语法错误一篇：`typst` 非 0 → `generate` 失败。

## 时间估算（参考）

- 首版脚本 + 文档：**0.5～1.5 天**（视对 Typst CLI 熟悉度）。
