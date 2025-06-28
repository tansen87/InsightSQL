# [Search](../src-tauri/src/lib/command/search.rs) - Match the corresponding row in a column

```
sample file (test.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
│  3  | hansen |
└─────┴────────┘
```



### 1. Equal

Set search criteria (Search mode: <u>Equal</u>, Search by column: <u>name</u>, Search rows: <u>tom|jerry</u>)

```
Equal search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
```



### 2. EqualMulti

Set search criteria (Search mode: <u>EqualMulti</u>, Search by column: <u>name</u>, Search rows: <u>tom|jerry</u>)

```
EqualMulti search result (2 output file: test_tom.csv, test_jerry.csv)
test_tom.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘

test_jerry.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
```



### 3. NotEqual

Set search criteria (Search mode: <u>NotEqual</u>, Search by column: <u>name</u>, Search rows: <u>tom|jerry</u>)

```
NotEqual search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
```



### 4. Contains

Set search criteria (Search mode: <u>Contains</u>, Search by column: <u>name</u>, Search rows: <u>om|jer</u>)

```
Contains search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
```



### 5. ContainsMulti

Set search criteria (Search mode: <u>ContainsMulti</u>, Search by column: <u>name</u>, Search rows: <u>om|jer</u>)

```
ContainsMulti search result (2 output file: test_om.csv, test_jer.csv)
test_om.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘

test_jer.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
```



### 6. NotContains

Set search criteria (Search mode: <u>NotContains</u>, Search by column: <u>name</u>, Search rows: <u>om|jer</u>)

```
NotContains search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
```



### 7. StartsWith

Set search criteria (Search mode: <u>StartsWith</u>, Search by column: <u>name</u>, Search rows: <u>to|jer</u>)

```
StartsWith search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
```



### 8. StartsWithMulti

Set search criteria (Search mode: <u>StartsWithMulti</u>, Search by column: <u>name</u>, Search rows: <u>to|jer</u>)

```
StartsWithMulti search result (2 output file: test_to.csv, test_jer.csv)
test_to.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘

test_jer.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
```



### 9. NotStartsWtih

Set search criteria (Search mode: <u>NotStartsWtih</u>, Search by column: <u>name</u>, Search rows: <u>to|jer</u>)

```
NotStartsWtih search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
```



### 10. EndsWtih

Set search criteria (Search mode: <u>EndsWtih</u>, Search by column: <u>name</u>, Search rows: <u>om|rry</u>)

```
EndsWtih search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
```



### 11. NotEndsWtih

Set search criteria (Search mode: <u>NotEndsWtih</u>, Search by column: <u>name</u>, Search rows: <u>om|rry</u>)

```
NotEndsWtih search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
```



### 12. Regex

Set search criteria (Search mode: <u>Regex</u>, Search by column: <u>name</u>, Search rows: <u>hansen</u>)

```
Regex search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
```

