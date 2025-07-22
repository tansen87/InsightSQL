# [Skip](../src-tauri/src/lib/cmd/skip.rs) - Skip rows form CSV
 
```
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
```


Set criteria (skip rows: <u>2</u>)
```
┌─────┬────────┐
│  2  │ jerry  │
├─────┼────────┤
│  3  | hansen |
└─────┴────────┘
```