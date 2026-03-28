// 文章入口模板：在 post/<slug>/index.typ 里 import 后调用 article。
// 图片等资源放在与 index.typ 同一目录，正文中使用 #image("foo.png")。
//
// 用法（title、date 为位置参数，正文为内容块）：
// #article("标题", "2026-03-27")[
//   正文…
// ]

#let article(title, date, body) = {
  set document(title: title)
  [= #title]
  text(size: 0.9em, fill: gray)[日期：#date]
  parbreak()
  body
}
