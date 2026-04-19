---
name: e3-sync
description: 同步 E3 講義和作業到 Obsidian，並用 AI 為新講義生成筆記
---

# E3 同步 Workflow

使用 `e3` Rust CLI（不再依賴 TS CLI）。Claude 手動執行每個步驟。

## Step 1: 列出課程並下載新講義

```bash
e3 courses --json          # 列出所有課程（取得 course ID）
e3 download <ID> --list    # 列出某課程的檔案
e3 download <ID> --type pdf -o "{vault}/{course}/slides" --skip-existing
```

對每個非排除課程執行 download。

## Step 2: 比對新講義 vs 現有筆記

掃描每個課程的 `slides/` 資料夾，比對同目錄下的 `.md` 筆記。
若 slide 沒有對應筆記（或筆記 < 300 bytes），標記為需要生成。

## Step 3: AI 生成筆記

讀 PDF slide，根據同課程既有筆記的格式風格，生成**完整**中文筆記。
用 subagent 平行處理多個筆記。

### 3a: 提取 slide 圖片到 assets/

PDF 裡的示意圖、公式圖、架構圖需要轉成 PNG 並放進筆記：

```bash
# 把 slide 每頁轉成 PNG（需要 poppler / pdftoppm，或 pymupdf）
python -c "
import fitz  # pip install pymupdf
import sys, os
pdf, outdir = sys.argv[1], sys.argv[2]
os.makedirs(outdir, exist_ok=True)
doc = fitz.open(pdf)
for i, page in enumerate(doc):
    pix = page.get_pixmap(dpi=150)
    pix.save(f'{outdir}/p{i+1}.png')
" "{slide.pdf}" "{vault}/{course}/assets/{chapter}"
```

然後在筆記裡 embed 需要的頁面：
```markdown
![說明文字](./assets/{chapter}/p{N}.png)
```

**判斷哪些頁要放圖**：
- 純文字條列 → 不放圖（筆記文字已涵蓋）
- 含**示意圖 / 流程圖 / 架構圖 / 截圖 / 範例輸出** → **必放原圖**
- 含公式推導過程 → 放圖（公式太複雜時）

### 3b: 用 excalidraw skill 自己畫圖

某些概念 slide 上是文字描述，但視覺化能大幅幫助理解（hierarchy、時序、拓撲、資料結構）。此時用 `excalidraw` skill 自製一張：

適合自己畫的場景：
- Memory hierarchy pyramid、protocol stack
- TCP / TLS / Kerberos handshake（sequence diagram）
- 網路攻擊拓撲（ARP spoofing、MITM）
- Pipeline（MIPS 5 stage、instruction flow）
- 資料結構（Bloom filter、cache mapping、radix tree）
- 演算法比較（FCFS / SSTF / SCAN disk scheduling）

不用自己畫的場景：
- slide 已有清楚示意圖 → 直接截圖
- 純 flat table → markdown table 就夠
- 簡單樹狀結構 → ASCII art 夠了

Excalidraw 檔案存到 `./assets/{chapter}/{name}.excalidraw.md`，在筆記裡用 `![[name.excalidraw|900]]` 嵌入。

### 3c: 筆記格式規則（嚴格遵守）

### 起手式：套 Templates/章節筆記.md

vault 內 `Templates/章節筆記.md` 是 stub 樣板：

```markdown
# {{title}}

> 課程：{{course}}
> 講義：{{slides}}

## 
```

新章節先用此 template 替換 `{{title}}` (chapter key)、`{{course}}` (vault folder name)、`{{slides}}` (slide 連結 `[[slides/X.pdf]]`)，再讓 AI 在 `## ` 後填充內容。**確保 stub 風格與既有筆記一致**。

### 其他可用 templates

| Template | 用途 |
|----------|------|
| `Templates/章節筆記.md` | 課程章節 stub（e3-sync 用）|
| `Templates/抽問記錄.md` | 每日抽問 log |
| `Templates/Cheat Sheet.md` | 考前重點整理 |
| `Templates/作業.md` | 作業繳交 |

- 語言：繁體中文，技術名詞保留英文
- 用 `==highlight==` 標重點、LaTeX `$...$` 寫公式
- `##` 大節、`###` 小節、bullet + table 混用
- 如果筆記內文需要用到 `$`，要用 `\$` 轉義，避免被誤認為公式開始
- **涵蓋 slide 所有內容，不跳過任何例子、細節、counter-example、注意事項、腳註**
  - 例子（如 "假設 page size = 4KB, block size = 64KB..."）**一定要完整照搬**，不可省略數值
  - 對比表格、優缺點、例外條件、常見誤解**全部要寫**
  - 寧可筆記長、不可漏。漏了一個細節 = 考試挖洞
  - 若 slide 有 quiz / review question，筆記要把題目 + 答案都記錄


### 各課程格式差異

| 課程 | Frontmatter | 標題格式 | 備註 |
|------|-------------|---------|------|
| Computer Organization | 無 | `# Ch{N}: {Title}` | 有 MIPS 範例、`---` 分節 |
| Computer Security Capstone | 無 | `# Ch{N} {Title}` | |
| GenAI Theory (515622) | 有 YAML (tags/slides/course/created) | `# 第{NN}堂、{Title}` | 含 `> 課程/講義` 引言 |
| Memory and Storage | 無 | `# L{N} {Title}` | |
| Network Planning (CCNA) | 無 | `# {原始檔名}` | |
| Network Security | 無 | `# {ID}. {Title}` | 含 `> 課程` 引言 |

## Step 4: 行事曆同步

同步 E3 作業截止日 + 手動事件到 Obsidian Calendar 和 ICS。

### 4a: 取得 E3 行事曆事件

```bash
e3 calendar --days 90 --json    # E3 API 的作業截止日
```

### 4b: 比對 Obsidian Calendar `.md` vs `calendar-events.json`

兩個資料源需雙向同步：

| 來源 | 位置 | 用途 |
|------|------|------|
| Obsidian Calendar | `{vault}/Calendar/YYYY-MM-DD {course} {title}.md` | Full Calendar plugin 顯示 |
| calendar-events.json | `~/Documents/e3-calendar/public/calendar-events.json` | 手動事件，CI merge 進 ICS |

掃描兩邊，找出缺漏：
- `calendar-events.json` 有但 Obsidian 沒有 → 建 `.md`
- E3 API 有新事件但兩邊都沒有 → 建 `.md`（E3 API 事件 ICS 由 CI 自動處理，不需加到 `calendar-events.json`）

### 4c: 建立缺少的 Obsidian Calendar `.md`

格式（Full Calendar plugin frontmatter）：

```markdown
---
title: {course} {name}
allDay: false
startTime: "HH:MM"
endTime: "HH:MM"
date: YYYY-MM-DD
completed: null
---

# {course} {name}
```

- 檔名：`YYYY-MM-DD {course} {name}.md`
- `completed: null`（未完成）、過期事件用 `completed: true`
- 若無時間資訊，用 `allDay: true` 並省略 startTime/endTime

### 4d: 若 `calendar-events.json` 有更動

```bash
cd ~/Documents/e3-calendar
git add public/calendar-events.json
git commit -m "update calendar events"
git push    # 觸發 CI → 重新生成 ICS → 上傳 R2
```

## Step 5: Git commit + push

在 vault 目錄 commit 新的 slides、筆記、和 Calendar，然後 push。
同時確認 e3-calendar repo 也已 push（若 Step 4d 有改動）。

## Vault 位置

`C:\Users\twsha\Documents\GitHub\note`

## 課程 ID 對照

| 課程 | ID | Vault 資料夾 |
|------|-----|-------------|
| 計算機組織 | 21330 | Computer Organization |
| 電腦安全總整 | 21347 | Computer Security Capstone |
| CCNA 網路規劃 | 21354 | Network Planning and Management Practices |
| 生成式AI概論 | 21364 | Introduction to Generative AI - From Theory to Application |
| 記憶體與儲存 | 21895 | Memory and Storage Systems |
| 網路安全實務 | 21917 | Network Security Practices-Attack and defense |
| GenAI 應用工程 | 23416 | (無教材) |

## 排除的課程

服務學習、高效能計算概論、日文二、Gender Equity
