---
name: e3-content
description: 瀏覽 E3 課程內容結構（sections、模組、附檔）
---

# E3 課程內容

```bash
e3 --json content <course-id>
```

列出課程的所有 sections 和模組（assignments、forums、resources、pages、URLs 等），包含附檔列表。

JSON 輸出：sections 陣列，每個 section 包含 `name` 和 `modules[]`。
每個 module 包含 `id`（cmid）、`name`、`modname`（assign/forum/resource/page/url/...）、`description`、`contents[]`（附檔）。

## 常見用途

- 找作業題目：篩選 `modname == "assign"` 的模組
- 找課程講義：篩選 `modname == "resource"` 或看 `contents[]`
- 找討論區 ID：篩選 `modname == "forum"` 取 `instance`

```bash
# 找課程中的所有作業
e3 --json content 21347 | jq '.data[].modules[] | select(.modname=="assign") | {id, name}'
```
