# [Transpose](../src-tauri/src/lib/cmd/transpose.rs) -

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


transpose result
```
sample file
┌─────┬─────┬──────┬───────┐
│ idx │ 1   │ 2    │ 3     │
├─────┼─────┼──────┼───────┤
│ name│ tom │ jerry│ hansen│
└─────┴─────┴──────┴───────┘
```