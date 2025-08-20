# [Sort](../src-tauri/src/lib/cmd/sort.rs) - Sorts CSV data lexicographically

```
sample file (The result of ExtSort same as Sort)
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ tom    │
│  1  │ jerry  │
│  3  | hansen |
└─────┴────────┘
```

#### 1.Set criteria (Select column: <u>idx</u>, Numeric: <u>true</u>, Reverse: <u>false</u>)

```
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ jerry  │
│  2  │ tom    │
│  3  | hansen |
└─────┴────────┘
```

#### 2.Set criteria (Select column: <u>idx</u>, Numeric: <u>true</u>, Reverse: <u>true</u>)

```
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
│  2  │ tom    │
│  1  | jerry  |
└─────┴────────┘
```

#### 3.Set criteria (Select column: <u>name</u>, Numeric: <u>false</u>, Reverse: <u>false</u>)

```
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  3  │ hansen │
│  1  │ jerry  │
│  2  | tom    |
└─────┴────────┘
```

#### 4.Set criteria (Select column: <u>name</u>, Numeric: <u>false</u>, Reverse: <u>true</u>)

```
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ tom    │
│  1  │ jerry  │
│  3  | hansen |
└─────┴────────┘
```