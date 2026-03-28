# Typst 博客实施建议报告

**执行顺序与细粒度排期** 以 [plan/schedule/00-index.md](../schedule/00-index.md) 为准：先 **01–04 后端（构建与 CI）**，再 **05–06 前端（站点壳与发布）**；**Hexo 式 CLI、站点配置、禁止 npm 构建** 见 [07-hexo-like-cli-and-config.md](../schedule/07-hexo-like-cli-and-config.md)。下文 4 周节奏可与各 schedule 文件对照，不必重复。

## 开发节奏建议（4 周）

### Week 1：MVP 构建打通

- 完成 `post/*.typ -> public/posts/*.html` 批量编译
- 建立基础模板与样例文章
- 固化目录与命名规范

### Week 2：站点可浏览

- 生成首页、文章列表、归档页
- 增加元信息读取与排序能力
- 固化 URL 与导航规则

### Week 3：工程化发布

- 接入 CI（构建、校验、预览）
- 建立 PR 门禁（构建失败/坏链阻断）
- 完成预发布验证与回滚预案

### Week 4：验收与扩展

- 输出上线验收清单
- 增加 RSS 与标签基础版
- 设计轻量搜索索引（JSON）

## 每周可交付物

- Week 1：构建脚本、目录模板、样例文章
- Week 2：`index`/列表/归档页面
- Week 3：CI 配置与回归用例
- Week 4：验收文档、RSS、标签页、搜索索引

## 质量门禁建议

- 全量文章可编译
- 重复文章 id 检测
- 站内链接无 404
- 关键元字段完整（title/date）
- 发布前 smoke test（首页 + 3 篇文章）

## 后续扩展建议

- 标签聚合页：`/tags/<tag>.html`
- RSS：`/rss.xml`（最近 20 篇）
- 搜索：`/search-index.json`
- 增量构建与体积告警
