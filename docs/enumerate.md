# [Enumerate](../src-tauri/src/lib/cmd/enumerate.rs) - Add a new column enumerating the lines of a CSV file

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


enumerate result
```
┌───────────────┬──────┬────────┐
│ enumerate_idx │ idx  │ name   │
├───────────────┼──────┼────────┤
│       1       │  1   │ tom    │
│       2       │  2   │ jerry  │
│       3       |  3   | hansen │
└───────────────┴──────┴────────┘
```