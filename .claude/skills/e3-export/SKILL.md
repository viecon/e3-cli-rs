---
name: e3-export
description: 匯出成績或作業為 CSV
---

```bash
e3 export grades -o grades.csv
e3 export assignments -o assignments.csv
```

CSV 帶 BOM（Excel 相容），JSON 輸出：`{ "success": true, "data": { "path", "rows" } }`
