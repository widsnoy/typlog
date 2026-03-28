# 03 后端：构建脚本

## 职责

- 扫描 `post/*.typ`（或约定子目录，若后续扩展）。
- 对每个源文件调用 `typst compile`（并传入 `meta.toml` 中的 title/date），目标为 `public/posts/<文章 id>/index.html`（目录规则见 02）。
- 构建前 **清理** `public/posts/` 或等价策略，避免幽灵页面。
- 将 Typst 版本、feature 开关写入一处（如环境变量文件或 CI 矩阵），避免散落。

## 命令形态（逻辑要求）

- **禁止**以 `package.json` / `npm run` 作为默认构建路径；须与 [07-hexo-like-cli-and-config.md](07-hexo-like-cli-and-config.md) 一致（如 `typlog generate`、或 `make generate`、或单一二进制）。
- 入口：仓库内 **只保留一种主入口**（文档与 CI 同一命令）。
- 参数：可选 `--clean`、可选 `--verbose`；默认行为与 CI 一致。

## 本地与 CI 一致性

- 同一命令在本地与 CI 应产生 **相同文件集合**（除时间戳类非确定性内容若存在则文档说明）。
- Windows/Linux 路径：脚本避免硬编码分隔符；若用 PowerShell，需在 README 说明。

## 验收清单

- [ ] 空 `post/`：构建失败或给出明确提示（二选一，需在文档写明）。
- [ ] 单篇成功：生成唯一 HTML。
- [ ] 多篇成功：篇数一致。
- [ ] 故意语法错误一篇：构建失败，日志含文件名。

## 时间估算（参考）

- 首版脚本 + 文档：**0.5～1.5 天**（视对 Typst CLI 熟悉度）。
