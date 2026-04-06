---
name: e3-calendar
description: 查看 E3 行事曆事件，產生 ICS 檔案訂閱到 iOS/Google Calendar
---

# E3 行事曆

```bash
e3 --json calendar
e3 --json calendar --days 30
e3 calendar --ics
e3 calendar --ics my-calendar.ics --ics-days 120
```

ICS 包含 E3 作業截止日 + `~/.calendar-events.json`（或 cwd 的 `calendar-events.json`）手動加的考試。
JSON 輸出：`{ "success": true, "data": [{ "id", "name", "timestart", "eventtype", "overdue", ... }] }`
