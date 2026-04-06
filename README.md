# E3 CLI (Rust)

NYCU E3 LMS 命令列工具 — Rust 重寫版。

零依賴部署、單一 binary、跨平台。19 個指令、36 個測試。

## 安裝

```bash
# 從 GitHub Release 下載（Linux/macOS/Windows）
# https://github.com/viecon/e3-cli-rs/releases

# 或本機編譯
cargo install --path crates/e3-cli
```

## 使用

```bash
# 登入
e3 login -u <student_id> -p <password>
e3 login --token <token>
e3 login --session <MoodleSession_cookie>

# 總覽
e3 whoami
e3 status

# 課程與作業
e3 courses
e3 assignments
e3 submission <id>
e3 grades [course_id]
e3 updates [course_id]

# 行事曆
e3 calendar
e3 calendar --ics out.ics --ics-days 120

# 公告與通知
e3 news
e3 notifications

# 檔案（按課程分資料夾下載）
e3 download <course_id> -o ./downloads
e3 download --all --skip-existing
e3 upload <assignment_id> file1.pdf

# 匯出 / 工具
e3 export grades -o grades.csv
e3 config list
e3 open [target]
e3 completions bash
```

## 全域選項

| 選項 | 說明 |
|------|------|
| `--json` | JSON 格式輸出（Agent 友好） |
| `--no-color` | 關閉顏色 |
| `--base-url <url>` | 覆蓋 E3 URL |

## JSON 輸出格式

```json
{ "success": true, "data": { ... } }
{ "success": false, "error": { "code": "...", "message": "..." } }
```

## Exit Codes

| Code | 意義 |
|------|------|
| 0 | 成功 |
| 1 | API / 一般錯誤 |
| 2 | 未認證 |
| 3 | 網路錯誤 |
| 4 | Session 過期 |

## 設定

- `~/.e3rc.json` — token、auth mode、排除設定
- `~/.e3.env` — 帳密（自動重新登入用）

## 開發

```bash
cargo build
cargo test          # 36 tests
cargo clippy
cargo fmt
```

## 關聯

- [e3-calendar](https://github.com/viecon/e3-calendar) — ICS 行事曆 (private, CI 用此 binary)
- [e3-mobile](https://github.com/viecon/e3-mobile) — PWA (e3.viecon.site)
- [e3-extension](https://github.com/viecon/e3-extension) — 瀏覽器 Extension
- [e3-cli](https://github.com/viecon/e3-cli) — TS 版 (legacy)

## License

MIT
