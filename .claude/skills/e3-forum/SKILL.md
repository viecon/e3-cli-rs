---
name: e3-forum
description: 瀏覽 E3 課程討論區和討論串
---

# E3 討論區

```bash
# 列出課程的所有討論區和討論串
e3 --json forum <course-id>

# 列出特定討論區的討論串
e3 --json forum <forum-id>

# 查看特定討論串的完整內容（所有貼文）
e3 --json forum <forum-id> --thread <discussion-id>

# 搜尋關鍵字（在所有貼文中搜尋）
e3 --json forum <course-id-or-forum-id> --search "keyword"
```

用 course ID 時會列出所有 forums 及其討論。
用 forum ID 時直接列出該 forum 的討論串。
加 `--thread` 會用 `mod_forum_get_discussion_posts` 取得完整貼文。
加 `--search` 會取得所有討論串的貼文，在 client 端做 keyword matching（case-insensitive）。

## JSON 輸出

課程/forum 模式：`{ "success": true, "data": [{ "forum_id", "forum_name", "discussions": [{ "id", "subject", "message", "userfullname", "numreplies", "timemodified" }] }] }`

Thread 模式：`{ "success": true, "data": { "posts": [{ "id", "subject", "message", "author": { "fullname" }, "timecreated", "parentid", "attachments" }] } }`

Search 模式：`{ "success": true, "data": { "keyword", "matches", "posts": [...] } }`

## 常見用途

- 看 TA 補充說明：assignment 常寫「discuss in Project Forum」
- 看同學提問和 TA 回覆
- 搜尋特定主題：`e3 forum 21353 --search "timeout"`
- 取得 discussion ID 後用 `--thread` 看完整對話
