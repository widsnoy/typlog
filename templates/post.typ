#set page(
  paper: "a4",
  margin: (x: 2.5cm, y: 2.5cm),
)
#set text(
  font: "New Computer Modern",
  lang: "zh",
  size: 11pt,
)
#set par(justify: true)
#set document(title: sys.inputs.at("title", default: ""))

// HTML 导出默认忽略公式；以下规则在网页中以内联 SVG 显示公式（PDF 等分页目标仍为默认数学排版）。
#show math.equation: it => {
  if target() == "html" {
    html.frame(it)
  } else {
    it
  }
}

在这里开始写正文。
