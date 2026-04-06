---
name: e3-grades
description: 查看 E3 成績
---

# E3 成績

```bash
# 所有課程成績總覽（一行一課）
e3 --json grades

# 特定課程詳細成績
e3 --json grades <course-id>

# 所有課程詳細成績（每門課的各項目分數）
e3 --json grades --all
```

`grades` 不帶參數：用 `gradereport_overview_get_course_grades` 取得課程級別總分。
`grades <course-id>`：用 `gradereport_user_get_grade_items` 取得該課的各項成績。
`grades --all`：對所有進行中的課程批次取得詳細成績。

JSON 輸出（--all）：`{ "success": true, "data": [{ "course_id", "course_shortname", "grades": [{ "itemname", "gradeformatted", "percentageformatted", ... }] }] }`
