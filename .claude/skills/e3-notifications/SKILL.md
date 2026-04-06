---
name: e3-notifications
description: 查看 E3 系統通知
---

```bash
e3 --json notifications
e3 --json notifications --limit 5
```

JSON 輸出：`{ "success": true, "data": [{ "id", "subject", "fullmessagehtml", "timecreated", "timeread", ... }] }`
