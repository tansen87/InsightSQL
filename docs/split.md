# [Split](../src-tauri/src/lib/cmd/split.rs) - Split one CSV file into many CSV files

### 1. Rows (standard csv file)
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
Set criteria (Split rows: <u>2</u>, Split mode: <u>Rows</u>)
```
split 1
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘

split 2
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  | hansen |
└─────┴────────┘
```


### 2. Lines
```
sample file
------------------------
idx,name
hello world...
say hello.
this is a test for lines.
------------------------
```
Set criteria (Split rows: <u>2</u>, Split mode: <u>Lines</u>)
```
split 1
------------------------
idx,name
hello world...
say hello.
------------------------

split 2
------------------------
idx,name
this is a test for lines.
------------------------
```


### [Index](../src-tauri/src/lib/cmd/idx.rs) - add index for csv