# [Count](../src-tauri/src/lib/cmd/count.rs) - Count the rows of CSV files

### 1. Count
```
Count result: 3
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
```


### 2. [Index](../src-tauri/src/lib/cmd/idx.rs) - add index for csv


### 3. Check - detecting issues caused by double quotation marks
```
Check result: 2
(This is an incorrect result)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │  tom   │
│  2  │ "jerry │
│  3  | hansen |
└─────┴────────┘
```