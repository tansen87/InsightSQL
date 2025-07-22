# [Pinyin](../src-tauri/src/lib/cmd/pinyin.rs) - Convert Chinese to Pinyin for specific column in CSV

```
sample file
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ 汤姆   │
│  2  │ 杰瑞   │
│  3  | hansen |
└─────┴────────┘
```


### upper
Set criteria (Select column: <u>name</u>, pinyin style: <u>upper</u>)
```
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ TANGMU │
│  2  │ JIERUI │
│  3  | hansen |
└─────┴────────┘
```


### lower
Set criteria (Select column: <u>name</u>, pinyin style: <u>lower</u>)
```
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tangmu │
│  2  │ jierui │
│  3  | hansen |
└─────┴────────┘
```