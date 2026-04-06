---
name: e3-status
description: E3 一鍵總覽：未繳作業、通知、課程數
---

# E3 總覽

```bash
e3 status
e3 --json status
```

一個指令顯示：未繳作業（含截止日）、未讀通知、課程數量。
JSON 輸出格式：`{ "success": true, "data": { "assignments": {...}, "notifications": {...}, "courses": {...} } }`
