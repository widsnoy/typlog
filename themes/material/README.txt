Material 主题资源说明
====================

turbo.min.js：Hotwired Turbo 8（与 crates/typlog-core 内 typlog init 内置字节一致），用于站内软导航；#typlog-bg 使用 data-turbo-permanent，切页时保留背景层。

主题默认已去掉卡片上的 backdrop-filter（滚动时极耗 GPU）；若仍卡，请把 background_blur_px 设为 0 或换预模糊图。

卡片「亚克力感」用半透明渐变实色透出全站 #typlog-bg + 极轻噪点 + 阴影（仍不用 backdrop-filter，避免滚动掉帧）。卡片透明度由 config.toml 的 glass_panel_opacity（0.35～1）控制，generate 时写入页面内 #typlog-glass-vars；与 background_opacity（整图背景层）不同。

快速滚动仍卡时：优先把 background_blur_px 设为 0（或换预模糊背景图），全屏 filter: blur() 在部分显卡上仍会拖慢合成。
当前版本下，当 background_blur_px > 0 且 background_image 是本地文件时，`typlog generate` 会自动输出 `public/background-preblur.png` 并改为使用该图（即前端不再实时 blur）。

预模糊背景（可选，减轻滚动开销）
--------------------------------
config.toml 里 background_blur_px > 0 时，浏览器会对全屏背景做实时 filter: blur()，在部分设备上仍可能掉帧。                     
可将大图预先模糊后放到站点根（与 generate 后的 public/ 一致），例如：

  magick background.webp -blur 0x4 background-blurred.webp

然后在 config.toml 中把 background_image 改为 background-blurred.webp，background_blur_px 设为 0。
