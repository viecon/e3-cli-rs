---
name: e3-news
description: 查看 E3 課程公告
---

# E3 公告

```bash
e3 --json news
e3 --json news --course <id>
e3 --json news --days 14
```

JSON 輸出：`{ "success": true, "data": [{ "course_id", "subject", "message", "userfullname", "timemodified" }] }`
