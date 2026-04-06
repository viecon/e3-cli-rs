---
name: e3-download
description: 下載 E3 課程講義
---

# E3 講義下載

```bash
# 列出檔案（不下載）
e3 --json download <course-id> --list

# 下載到指定目錄（按課程分資料夾）
e3 download <course-id> -o ./downloads

# 只下載 PDF
e3 download <course-id> --type pdf -o ./downloads

# 下載所有課程
e3 download --all -o ./downloads --skip-existing
```

用 `e3 --json courses` 查課程 ID。REST API 優先，session 模式自動 fallback 到 HTML 爬取。
下載結構：`downloads/{course_shortname}/file.pdf`
