# [Split-string](../src-tauri/src/lib/cmd/string/split.rs)

```
sample file
┌─────┬──────────┐
│ idx │ name     │
├─────┼──────────┤
│  1  │ tom-1    │
│  2  │ jerry-2  │
│  3  | hansen-3 |
└─────┴──────────┘
```


### 1. Nth
Set criteria (Select column: <u>name</u>, Number of the string: <u>1</u>, String separator: <u>-</u>, String mode: <u>Nth</u>)
```
┌─────┬──────────┬──────────┐
│ idx │ name     │ name_nth │
├─────┼──────────┼──────────┤
│  1  │ tom-1    │  tom     │
│  2  │ jerry-2  │  jerry   │
│  3  | hansen-3 |  hansen  │
└─────┴──────────┴──────────┘
```


### 2. Nmax
Set criteria (Select column: <u>name</u>, Number of the string: <u>2</u>, String separator: <u>-</u>, String mode: <u>Nmax</u>)
```
┌─────┬──────────┬────────────┬────────────┐
│ idx │ name     │ name_nmax1 │ name_nmax2 │
├─────┼──────────┼────────────┼────────────┤
│  1  │ tom-1    │  tom       │     1      │
│  2  │ jerry-2  │  jerry     │     2      │
│  3  | hansen-3 |  hansen    │     3      │
└─────┴──────────┴────────────┴────────────┘
```