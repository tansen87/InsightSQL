# [Select](../src-tauri/src/lib/cmd/select.rs) - Select, re-order, drop columns

```
sample file
┌─────┬────────┬─────────────┐
│ idx │ name   │ _filename_  │
├─────┼────────┼─────────────┤
│  1  │ tom    │ test1.csv   │
│  2  │ jerry  │ test1.csv   │
│  3  | hansen | test1.csv   │
└─────┴────────┴─────────────┘
```


### 1. Include
Set criteria (Select column: <u>name,idx</u>, Select mode: <u>Include</u>)
```
┌────────┬─────┐
│ name   │ idx │
├────────┼─────┤
│ tom    │  1  │
│ jerry  │  2  │
│ hansen |  3  |
└────────┴─────┘
```


### 2. Exclude
Set criteria (Select column: <u>_filename_</u>, Select mode: <u>Exclude</u>)
```
sample file
┌─────┬────────┬─────────────┐
│ idx │ name   │ _filename_  │
├─────┼────────┼─────────────┤
│  1  │ tom    │ test1.csv   │
│  2  │ jerry  │ test1.csv   │
│  3  | hansen | test1.csv   │
└─────┴────────┴─────────────┘
```