---
name: e3-config
description: 管理 E3 助手設定（vault 路徑、排除課程等）
---

```bash
e3 config list
e3 config get vault_path
e3 config set vault_path "/path/to/vault"
e3 config set excluded_courses '["服務學習","日文","Gender Equity"]'
e3 config set excluded_extensions '["mp4","mkv","avi","pkt"]'
```

設定檔：`~/.e3rc.json`（snake_case，也接受 camelCase 讀取）
