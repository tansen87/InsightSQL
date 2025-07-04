# [Slice](../src-tauri/src/lib/cmd/slice.rs) - Slicing of CSV column

```
sample file (test.csv)
┌─────┬──────────┐
│ idx │ name     │
├─────┼──────────┤
│  1  │ tom-1    │
│  2  │ jerry-2  │
│  3  | hansen-3 |
└─────┴──────────┘
```


### 1. Left

Set slice criteria (Slice by column: <u>name</u>, Number of the slice: <u>3</u>, Slice mode: <u>Left</u>)

```
Left slice result (test.slice.csv)
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_nchar │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │    tom     │
│  2  │ jerry-2  │    jer     │
│  3  | hansen-3 |    han     │
└─────┴──────────┴────────────┘
```


### 2. Right

Set slice criteria (Slice by column: <u>name</u>, Number of the slice: <u>3</u>, Slice mode: <u>Right</u>)

```
Right slice result (test.slice.csv)
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_nchar │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │    m-1     │
│  2  │ jerry-2  │    y-2     │
│  3  | hansen-3 |    n-3     │
└─────┴──────────┴────────────┘
```


### 3. StartLength

Set slice criteria (Slice by column: <u>name</u>, Start index: <u>1</u>, Length of the slice: <u>3</u>, Slice mode: <u>StartLength</u>)

```
StartLength slice result (test.slice.csv)
┌─────┬──────────┬─────────┐
│ idx │ name     │ name_sl │
├─────┼──────────┼─────────┤
│  1  │ tom-1    │   tom   │
│  2  │ jerry-2  │   jer   │
│  3  | hansen-3 |   han   │
└─────┴──────────┴─────────┘
```

Set slice criteria (Slice by column: <u>name</u>, Start index: <u>-1</u>, Length of the slice: <u>3</u>, Slice mode: <u>StartLength</u>)

```
StartLength slice result (test.slice.csv)
┌─────┬──────────┬─────────┐
│ idx │ name     │ name_sl │
├─────┼──────────┼─────────┤
│  1  │ tom-1    │   m-1   │
│  2  │ jerry-2  │   y-2   │
│  3  | hansen-3 |   n-3   │
└─────┴──────────┴─────────┘
```


### 4. Nth

Set slice criteria (Slice by column: <u>name</u>, Number of the slice: <u>1</u>, Slice separator: <u>-</u>, Slice mode: <u>Nth</u>)

```
Nth slice result (test.slice.csv)
┌─────┬──────────┬──────────┐
│ idx │ name     │ name_nth │
├─────┼──────────┼──────────┤
│  1  │ tom-1    │  tom     │
│  2  │ jerry-2  │  jerry   │
│  3  | hansen-3 |  hansen  │
└─────┴──────────┴──────────┘
```


### 5. Nmax

Set slice criteria (Slice by column: <u>name</u>, Number of the slice: <u>2</u>, Slice separator: <u>-</u>, Slice mode: <u>Nmax</u>)

```
Nth slice result (test.slice.csv)
┌─────┬──────────┬────────────┬────────────┐
│ idx │ name     │ name_nmax1 │ name_nmax2 │
├─────┼──────────┼────────────┼────────────┤
│  1  │ tom-1    │  tom       │     1      │
│  2  │ jerry-2  │  jerry     │     2      │
│  3  | hansen-3 |  hansen    │     3      │
└─────┴──────────┴────────────┴────────────┘
```