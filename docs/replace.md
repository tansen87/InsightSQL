# [Replace](../src-tauri/src/lib/cmd/replace.rs) - Replace CSV data using a regex

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


Set criteria (Select column: <u>name</u>, regex pattern: <u>tom</u>, replacement: <u>guy</u>)
```
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ guy    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
```