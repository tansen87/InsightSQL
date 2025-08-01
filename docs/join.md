# [Join](../src-tauri/src/lib/cmd/join.rs) - Joins two sets of CSV data on the specified columns

```
sample file
left.csv           right.csv
┌─────┬────────┐   ┌─────┬──────┐
│ idx │ name   │   │ idx │ age  │
├─────┼────────┤   ├─────┼──────┤
│  1  │ tom    │   │  1  │ 20   │
│  2  │ jerry  │   │  3  | 18   |
│  3  | hansen |   │  5  │ 10   |
└─────┴────────┘   └─────┴──────┘
```


### inner
```
inner join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  3  │ hansen │  3  | 18   |
└─────┴────────┴─────┴──────┘
```


### left
```
left outer join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  2  │ jerry  |     |      |
│  3  | hansen |  3  │ 18   |
└─────┴────────┴─────┴──────┘
```


### right
```
right outer join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  3  │ hansen │  3  | 18   |
│     |        │  5  │ 10   |
└─────┴────────┴─────┴──────┘
```


### full
```
full outer join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  2  │ jerry  │     |      |
│  3  | hansen |  3  │ 18   |
│     |        │  5  │ 10   |
└─────┴────────┴─────┴──────┘
```


### cross
```
cross join result (left_on='idx', right_on='idx')
┌─────┬────────┬─────┬──────┐
│ idx │ name   │ idx │ age  │
├─────┼────────┼─────┼──────┤
│  1  │ tom    │  1  │ 20   │
│  1  │ tom    │  3  │ 18   │
│  1  │ tom    │  5  │ 10   │
│  2  │ jerry  │  1  | 20   |
│  2  │ jerry  │  3  | 18   |
│  2  │ jerry  │  5  | 10   |
│  3  | hansen |  1  │ 20   |
│  3  | hansen |  3  │ 18   |
│  3  | hansen |  5  │ 10   |
└─────┴────────┴─────┴──────┘
```


### left semi
```
left semi join result (left_on='idx', right_on='idx')
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  1  │ tom    │
│  3  | hansen |
└─────┴────────┘
```


### left anti
```
left anti join result (left_on='idx', right_on='idx')
┌─────┬────────┐
│ idx │ name   │
├─────┼────────┤
│  2  │ jerry  │
└─────┴────────┘
```


### right semi
```
right semi join result (left_on='idx', right_on='idx')
┌─────┬──────┐
│ idx │ age  │
├─────┼──────┤
│  5  │ 10   |
└─────┴──────┘
```


### right anti
```
right anti join result (left_on='idx', right_on='idx')
┌─────┬──────┐
│ idx │ age  │
├─────┼──────┤
│  1  │ 20   │
│  3  | 18   |
└─────┴──────┘
```