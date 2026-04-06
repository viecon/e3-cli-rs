---
name: e3-courses
description: 列出 NYCU E3 的所有選修課程
---

# E3 課程列表

```bash
e3 --json courses
e3 --json courses --all
```

JSON 輸出：`{ "success": true, "data": [{ "id", "shortname", "fullname", "progress", ... }] }`
加 `--all` 列出所有課程含已結束的。
