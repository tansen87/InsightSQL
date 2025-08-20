# [Apply](../src-tauri/src/lib/cmd/skip.rs) - Apply series of string, math transformations to given CSV column/s

### Operations
##### 1.Copy
```
(Select column: idx)
┌─────┬────────┬─────────┐
│ idx │ name   │ idx_new │
├─────┼────────┼─────────┤
│  1  │ tom    │    1    │
│  2  │ jerry  │    2    │
│  3  | hansen |    3    │
└─────┴────────┴─────────┘
```

##### 2.Len
```
(Select column: name)
┌─────┬────────┬──────────┐
│ idx │ name   │ name_new │
├─────┼────────┼──────────┤
│  1  │ tom    │    3     │
│  2  │ jerry  │    5     │
│  3  | hansen |    5     │
└─────┴────────┴──────────┘
```

##### 3.Lower
```
(Select column: name)
┌─────┬────────┬──────────┐
│ idx │ name   │ name_new │
├─────┼────────┼──────────┤
│  1  │ tom    │  tom     │
│  2  │ jerry  │  jerry   │
│  3  | hansen |  hansen  │
└─────┴────────┴──────────┘
```

##### 4.Upper
```
(Select column: name)
┌─────┬────────┬──────────┐
│ idx │ name   │ name_new │
├─────┼────────┼──────────┤
│  1  │ tom    │  TOM     │
│  2  │ jerry  │  JERRY   │
│  3  | hansen |  HANSEN  │
└─────┴────────┴──────────┘
```

##### 5.Trim
```
(Select column: name)
┌─────┬─────────────┬──────────┐
│ idx │ name        │ name_new │
├─────┼─────────────┼──────────┤
│  1  │ \n tom      │  tom     │
│  2  │ jerry\n     │  jerry   │
│  3  | \n hansen\n |  hansen  │
└─────┴─────────────┴──────────┘
```

##### 6.Ltrim
```
(Select column: name)
┌─────┬─────────────┬──────────┐
│ idx │ name        │ name_new │
├─────┼─────────────┼──────────┤
│  1  │ \n tom      │  tom     │
│  2  │ jerry\n     │  jerry\n │
│  3  | \n hansen\n |  hansen\n│
└─────┴─────────────┴──────────┘
```

##### 7.Rtrim
```
(Select column: name)
┌─────┬─────────────┬──────────┐
│ idx │ name        │ name_new │
├─────┼─────────────┼──────────┤
│  1  │ \n tom      │ \n tom   │
│  2  │ jerry\n     │  jerry   │
│  3  | \n hansen\n | \n hansen│
└─────┴─────────────┴──────────┘
```

##### 8.Replace
```
(Select column: name, from: tom, to: TTT)
┌─────┬────────┬──────────┐
│ idx │ name   │ name_new │
├─────┼────────┼──────────┤
│  1  │ tom    │  TTT     │
│  2  │ jerry  │  jerry   │
│  3  | hansen |  hansen  │
└─────┴────────┴──────────┘
```

##### 9.Round
```
(Select column: idx)
┌───────┬────────┬─────────┐
│ idx   │ name   │ idx_new │
├───────┼────────┼─────────┤
│ 1.234 │ tom    │  1.23   │
│ 2     │ jerry  │  2.00   │
│ 3.556 | hansen |  3.56   │
└───────┴────────┴─────────┘
```

##### 9.Squeeze
```
(Select column: idx)
┌─────┬───────────┬──────────┐
│ idx │ name      │ name_new │
├─────┼───────────┼──────────┤
│  1  │ tom\s\s   │  tom\s   │
│  2  │ \s\s jerry│  \s jerry│
│  3  | hansen    |  hansen  │
└─────┴───────────┴──────────┘
```

##### 10.Strip
```
(Select column: idx)
┌─────┬───────────┬──────────┐
│ idx │ name      │ name_new │
├─────┼───────────┼──────────┤
│  1  │ \n tom\r  │  tom     │
│  2  │ \r\n jerry│  jerry   │
│  3  | hansen\r\n|  hansen  │
└─────┴───────────┴──────────┘
```

##### 11.Reverse
```
(Select column: name)
┌─────┬────────┬──────────┐
│ idx │ name   │ name_new │
├─────┼────────┼──────────┤
│  1  │ tom    │  mot     │
│  2  │ jerry  │  yrrej   │
│  3  | hansen |  nesnah  │
└─────┴────────┴──────────┘
```

##### 12.Abs
```
(Select column: idx)
┌─────┬────────┬─────────┐
│ idx │ name   │ idx_new │
├─────┼────────┼─────────┤
│ -1  │ tom    │    1    │
│  2  │ jerry  │    2    │
│ -3  | hansen |    3    │
└─────┴────────┴─────────┘
```

##### 13.Neg
```
(Select column: idx)
┌─────┬────────┬─────────┐
│ idx │ name   │ idx_new │
├─────┼────────┼─────────┤
│ -1  │ tom    │    1    │
│  2  │ jerry  │   -2    │
│ -3  | hansen |    3    │
└─────┴────────┴─────────┘
```


### Cat
```
(formatstr: {idx}-{name})
┌─────┬────────┬─────────┐
│ idx │ name   │   idx   │
├─────┼────────┼─────────┤
│  1  │ tom    │ 1-tom   │
│  2  │ jerry  │ 2-jerry │
│  3  | hansen | 3-hansen│
└─────┴────────┴─────────┘
```


### CalcConv
```
(formatstr: {idx}+{idx2})
┌─────┬──────┬─────┐
│ idx │ idx2 │ idx │
├─────┼──────┼─────┤
│  1  │  2   │  3  │
│  2  │  3   │  5  │
│  3  |  5   |  8  │
└─────┴──────┴─────┘
```