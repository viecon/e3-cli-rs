# E3 CLI (Rust)

NYCU E3 LMS 命令列工具 — Rust 重寫版。

零依賴部署、單一 binary、跨平台。

## 安裝

```bash
# 從 GitHub Release 下載
# 或
cargo install --path crates/e3-cli
```

## 使用

```bash
# 登入
e3 login -u <student_id>
e3 login --token <token>
e3 login --session <MoodleSession_cookie>

# 查看狀態
e3 whoami
e3 status

# 課程與作業
e3 courses
e3 assignments
e3 submission <assignment_id>
e3 grades [course_id]

# 行事曆
e3 calendar
e3 calendar --ics           # 產生 ICS 檔案
e3 calendar --ics out.ics   # 指定輸出路徑

# 公告與通知
e3 news
e3 notifications

# 檔案操作
e3 download <course_id>
e3 download --all
e3 upload <assignment_id> file1.pdf file2.zip

# 匯出
e3 export grades
e3 export assignments

# 工具
e3 config list
e3 config set vault_path /path/to/vault
e3 open
e3 open <course_id>
e3 completions bash
```

## 全域選項

| 選項 | 說明 |
|------|------|
| `--json` | JSON 格式輸出 |
| `--no-color` | 關閉顏色 |
| `--base-url <url>` | 覆蓋 E3 URL |

## JSON 輸出格式

```json
{ "success": true, "data": { ... } }
{ "success": false, "error": { "code": "...", "message": "..." } }
```

## 設定

- `~/.e3rc.json` — token、設定
- `~/.e3.env` — 帳密（自動重新登入用）

## 開發

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

## License

MIT
