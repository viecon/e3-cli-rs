---
name: e3-fetch
description: 用已認證的 session 抓取 E3 頁面內容
---

# E3 頁面抓取

```bash
e3 --json fetch "<url>"
e3 fetch "<url>"
```

用已登入的 token/session 存取任意 E3 頁面，擷取 `#region-main` 的內容並轉成純文字。

JSON 輸出：`{ "success": true, "data": { "url", "content" (純文字), "html" (原始 HTML) } }`

## 常見用途

- 抓作業完整說明（Moodle page 或 assignment description）
- 讀取需要登入才能看的頁面
- defuddle / WebFetch 無法存取 E3 時的替代方案

```bash
# 抓作業頁面
e3 --json fetch "https://e3p.nycu.edu.tw/mod/assign/view.php?id=192787"

# 抓課程首頁
e3 --json fetch "https://e3p.nycu.edu.tw/course/view.php?id=21347"

# 抓 Moodle page
e3 --json fetch "https://e3p.nycu.edu.tw/mod/page/view.php?id=12345"
```
