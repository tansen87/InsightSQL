# [Rename](../src-tauri//src/lib/cmd/rename.rs) - Rename the columns of a CSV

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


Set new headers: idx1, name1
```
┌──────┬────────┐
│ idx1 │ name1  │
├──────┼────────┤
│  1   │ tom    │
│  2   │ jerry  │
│  3   | hansen |
└──────┴────────┘
```