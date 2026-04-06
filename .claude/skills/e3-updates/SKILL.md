---
name: e3-updates
description: 查看課程最近更新（新教材、設定變更、成績等）
---

```bash
e3 --json updates
e3 --json updates <course-id> --days 14
```

JSON 輸出：`{ "success": true, "data": [{ "course_id", "update_type", "time", "item_count" }] }`
