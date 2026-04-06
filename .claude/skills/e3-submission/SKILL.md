---
name: e3-submission
description: 查看作業提交詳情、已上傳檔案、成績與回饋
---

# E3 作業提交詳情

```bash
# 查看提交狀態
e3 --json submission <cmid>

# 下載已繳交的檔案
e3 submission <cmid> --download
e3 submission <cmid> --download -o ./downloads
```

`<cmid>` 是 assignment 的 course module ID（不是 instance ID）。

JSON 輸出：`{ "success": true, "data": { "lastattempt": { "submission": { "status", "timemodified", "plugins" }, ... }, "feedback": { "gradefordisplay", "gradeddate" } } }`

## 如何取得 cmid

1. 從 `e3 --json assignments` 的結果取 `cmid` 欄位（未繳作業）
2. 從 `e3 --json assignments --course <id> --all` 找已繳交的作業
3. 從 `e3 --json content <course-id>` 找 modname=="assign" 的模組 `id`

## 下載已繳交的檔案

```bash
# 下載到當前目錄
e3 submission 195931 --download

# 下載到指定目錄
e3 submission 195931 --download -o ./my-submissions
```

`--download` 會從 `plugins[].fileareas[].files[]` 取出所有已上傳檔案並下載。
