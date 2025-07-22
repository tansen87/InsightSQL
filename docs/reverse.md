# [Reverse](../src-tauri/src/lib/cmd/reverse.rs) - Reverse order of rows in a CSV

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


reverse result
```
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
│  2  │ jerry  │
│  1  | tom    |
└─────┴────────┘
```