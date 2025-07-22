# [Fill](../src-tauri/src/lib/cmd/fill.rs) - Fill empty fields in selected columns of a CSV

```
sample file
┌─────┬─────────┐
│ idx │ name    │
├─────┼─────────┤
│  1  │         │
│  2  │ jerry   │
│  3  |         |
└─────┴─────────┘
```


### 1. fill
Set criteria (Select column: <u>name</u>, fill mode: <u>fill</u>, fill value: <u>jerry</u>)
```
┌─────┬─────────┐
│ idx │ name    │
├─────┼─────────┤
│  1  │ jerry   │
│  2  │ jerry   │
│  3  | jerry   |
└─────┴─────────┘
```


### 2. f-fill
Set criteria (Select column: <u>name</u>, fill mode: <u>f-fill</u>)
```
┌─────┬─────────┐
│ idx │ name    │
├─────┼─────────┤
│  1  │         │
│  2  │ jerry   │
│  3  | jerry   |
└─────┴─────────┘
```