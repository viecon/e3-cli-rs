---
name: e3-news
description: 查看 E3 課程公告
---

# E3 公告

```bash
# 所有課程的公告（最近 7 天）
e3 --json news

# 特定課程（positional 或 --course 皆可）
e3 --json news 21347
e3 --json news --course 21347

# 自訂天數
e3 --json news 21347 --days 14
```

JSON 輸出：`{ "success": true, "data": [{ "course_id", "subject", "message", "userfullname", "timemodified" }] }`
