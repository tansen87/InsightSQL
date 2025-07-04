# [Cat](../src-tauri/src/lib/cmd/cat.rs) - Merge multiple CSV or Excel files into one CSV or xlsx file

```
sample file
test1.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘

test2.xlsx (test2.csv)
┌─────┬─────┐
│ age │ idx │
├─────┼─────┤
│ 10  │  4  │
│ 21  │  5  │
│ 31  |  6  |
└─────┴─────┘
```

### 1. Polars (support csv and excel file)

```
┌─────┬────────┬─────────────┬─────┐
│ idx │ name   │ _filename_  │ age │
├─────┼────────┼─────────────┼─────┤
│  1  │ tom    │ test1.csv   │     │
│  2  │ jerry  │ test1.csv   │     │
│  3  | hansen | test1.csv   │     │
│  4  │        │ test2.xlsx  │ 10  │
│  5  │        │ test2.xlsx  │ 21  │
│  6  │        │ test2.xlsx  │ 31  │
└─────┴────────┴─────────────┴─────┘
```

### 2. CSV (only support csv file)

```
┌─────┬────────┬─────┐
│ idx │ name   │ age │
├─────┼────────┤─────┤
│  1  │ tom    │     │
│  2  │ jerry  │     │
│  3  | hansen |     │
│  4  │        │ 10  │
│  5  │        │ 21  │
│  6  │        │ 31  │
└─────┴────────┴─────┘
```

### 3. Duplicate (only support csv file)

```
sample file (test.csv)
┌──────┬───────┐
│ name │ name  │
├──────┼───────┤
│  1   │ tom   │
└──────┴───────┘

duplicate result: {"name"}
```

