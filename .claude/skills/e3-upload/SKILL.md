---
name: e3-upload
description: 上傳檔案到 E3 並提交作業
---

# E3 作業上傳

```bash
# 上傳並提交
e3 upload <assignment-id> file1.pdf file2.zip

# 只上傳不提交
e3 upload <assignment-id> file1.pdf --no-submit
```

用 `e3 --json assignments` 查 assignment ID。
JSON 輸出：`{ "success": true, "data": { "item_id", "submitted", "files" } }`
