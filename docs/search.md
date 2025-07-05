# [Search](../src-tauri/src/lib/cmd/search.rs) - Match the corresponding row in a column

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

Set search criteria (Select column: <u>name</u>, Search mode: <u>Equal</u>, Search conditions: <u>tom|jerry</u>)

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

Set search criteria (Select column: <u>name</u>, Search mode: <u>EqualMulti</u>, Search conditions: <u>tom|jerry</u>)

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

Set search criteria (Select column: <u>name</u>, Search mode: <u>NotEqual</u>, Search conditions: <u>tom|jerry</u>)

```
NotEqual search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
```

### 4. Contains

Set search criteria (Select column: <u>name</u>, Search mode: <u>Contains</u>, Search conditions: <u>om|jer</u>)

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

Set search criteria (Select column: <u>name</u>, Search mode: <u>ContainsMulti</u>, Search conditions: <u>om|jer</u>)

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

Set search criteria (Select column: <u>name</u>, Search mode: <u>NotContains</u>, Search conditions: <u>om|jer</u>)

```
NotContains search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
```

### 7. StartsWith

Set search criteria (Select column: <u>name</u>, Search mode: <u>StartsWith</u>, Search conditions: <u>to|jer</u>)

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

Set search criteria (Select column: <u>name</u>, Search mode: <u>StartsWithMulti</u>, Search conditions: <u>to|jer</u>)

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

### 9. NotStartsWith

Set search criteria (Select column: <u>name</u>, Search mode: <u>NotStartsWith</u>, Search conditions: <u>to|jer</u>)

```
NotStartsWith search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
```

### 10. EndsWith

Set search criteria (Select column: <u>name</u>, Search mode: <u>EndsWith</u>, Search conditions: <u>om|rry</u>)

```
EndsWith search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
```

### 11. EndsWithMulti

Set search criteria (Select column: <u>name</u>, Search mode: <u>EndsWithMulti</u>, Search conditions: <u>om|rry</u>)

```
EndsWithMulti search result (2 output file: test_om.csv, test_rry.csv)
test_om.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
└─────┴────────┘

test_rry.csv
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
```

### 12. NotEndsWith

Set search criteria (Select column: <u>name</u>, Search mode: <u>NotEndsWith</u>, Search conditions: <u>om|rry</u>)

```
NotEndsWith search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
└─────┴────────┘
```

### 13. Regex

Set search criteria (Select column: <u>name</u>, Search mode: <u>Regex</u>, Search conditions: <u>hansen</u>)

```
Regex search result (1 output file: test_search.csv)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  2  │ jerry  │
└─────┴────────┘
```

