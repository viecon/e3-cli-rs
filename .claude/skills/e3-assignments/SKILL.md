---
name: e3-assignments
description: 查看 E3 未完成作業
---

# E3 作業列表

```bash
# 未繳作業（預設 30 天內）
e3 --json assignments
e3 --json assignments --days 90

# 特定課程的所有作業（含已繳交）
e3 --json assignments --course <course-id> --all

# 特定課程的未繳作業
e3 --json assignments --course <course-id>

# 所有課程的所有作業
e3 --json assignments --all
```

用 calendar API 取得未繳作業（actionable=true 的 assignment 事件）。
加 `--course` 改用 REST API（`mod_assign_get_assignments`），資訊更完整。
加 `--all` 會包含已繳交的作業。

JSON 輸出：`{ "success": true, "data": [{ "id", "cmid", "course_id", "course_shortname", "name", "duedate", "intro", "submission_status", "is_overdue", "description", "attachments" }] }`

## 找已繳交作業的 ID

如果要找某課程已繳交的作業 cmid（例如要看提交詳情或下載檔案）：
```bash
e3 --json assignments --course <course-id> --all
```
從結果中找 `submission_status == "submitted"` 的項目，取其 `cmid`。
