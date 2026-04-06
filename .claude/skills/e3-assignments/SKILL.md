---
name: e3-assignments
description: 查看 E3 未完成作業
---

# E3 未完成作業

```bash
e3 --json assignments
e3 --json assignments --days 90
```

用 calendar API 取得未繳作業（actionable=true 的 assignment 事件）。
JSON 輸出：`{ "success": true, "data": [{ "id", "cmid", "course_id", "course_shortname", "name", "duedate", "submission_status", "is_overdue" }] }`
