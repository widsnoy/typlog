# 02 后端：目录与元数据契约

**实现状态**：与 `typlog-core`（`meta`、`post`、`scaffold`、`build`）一致；**已实现**。

## 目录约定

- `post/<文章 id>/`：每篇一个目录；**必填** `meta.toml`（标题、日期、草稿等）与 `index.typ`（正文）。`文章 id` 为小写 kebab-case 目录名，用作 `public/posts/<文章 id>/` 的 URL 路径段。
- `public/posts/`：**仅构建产物**，不手改；可纳入 `.gitignore` 或仅 CI 生成（团队策略二选一，需在仓库说明）。
- `templates/`（或等价）：**默认** `post.typ`（写入新文章时为 `index.typ`）为**自包含**版式——`#set page`（纸张、边距）、`#set text`（字体、字号、语言）等，不依赖单独的封装文件。若项目需要可复用片段，仍可在此目录放可被 `#import` 的 `.typ`；**不**与前端页面模板混名，避免混淆。
- **HTML 导出**：Typst 的 `html` 格式下，`#set page` 等页面规则可能被忽略（以 Typst 版本行为为准）；默认模板仍写入这些规则，便于日后导出 PDF 或 Typst 行为更新后一致。

## 命名与文章 id

- 文章目录名：`kebab-case`（如 `hello-world`），与 URL 路径段一致。
- **禁止**：同一文章 id 两个目录；构建脚本应 **检测重复并失败**。

## 元数据（最小集）

在 **`meta.toml`** 中声明（单一事实来源）；`typlog generate` 通过 `typst compile --input title=… date=…` 注入 `sys.inputs`，`index.typ` 内可用其设置 `#set document(title: …)` 等，**不必**在正文中重复抄写标题与日期。

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `title` | 是 | 文章标题 |
| `date` | 是 | 建议 `YYYY-MM-DD`，用于排序 |
| `draft` | 否 | 为真则默认不参与发布列表与全量编译 |

## 资源引用

- 图片等静态资源：约定目录（如 `public/assets/` 或 `post/<文章 id>/` 下与 `index.typ` 同级），并在本文档写清 **相对路径基准**（编译工作目录、root 参数）。
- 后端阶段验收：**至少 1 篇** 带本地图片的文章可成功编译且资源在部署后可达（路径规则在 03 与 05 衔接）。

## 产出

- 一份 `README` 片段或 `docs/content-contract.md`：**仅描述后端契约**，供贡献者添加文章时查阅。
