# [Slice](../src-tauri/src/lib/cmd/slice.rs) - Slicing of CSV column

```
sample file
┌─────┬──────────┐
│ idx │ name     │
├─────┼──────────┤
│  1  │ tom-1    │
│  2  │ jerry-2  │
│  3  | hansen-3 |
└─────┴──────────┘
```


### 1. Left

Set criteria (Select column: <u>name</u>, Number of the string: <u>3</u>, String mode: <u>Left</u>)

```
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_nchar │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │    tom     │
│  2  │ jerry-2  │    jer     │
│  3  | hansen-3 |    han     │
└─────┴──────────┴────────────┘
```


### 2. Right

Set criteria (Select column: <u>name</u>, Number of the string: <u>3</u>, String mode: <u>Right</u>)

```
┌─────┬──────────┬────────────┐
│ idx │ name     │ name_nchar │
├─────┼──────────┼────────────┤
│  1  │ tom-1    │    m-1     │
│  2  │ jerry-2  │    y-2     │
│  3  | hansen-3 |    n-3     │
└─────┴──────────┴────────────┘
```


### 3. StartLength

Set criteria (Select column: <u>name</u>, Start index: <u>1</u>, Length of the string: <u>3</u>, String mode: <u>StartLength</u>)

```
┌─────┬──────────┬─────────┐
│ idx │ name     │ name_sl │
├─────┼──────────┼─────────┤
│  1  │ tom-1    │   tom   │
│  2  │ jerry-2  │   jer   │
│  3  | hansen-3 |   han   │
└─────┴──────────┴─────────┘
```

Set criteria (Select column: <u>name</u>, Start index: <u>-1</u>, Length of the string: <u>3</u>, String mode: <u>StartLength</u>)

```
┌─────┬──────────┬─────────┐
│ idx │ name     │ name_sl │
├─────┼──────────┼─────────┤
│  1  │ tom-1    │   m-1   │
│  2  │ jerry-2  │   y-2   │
│  3  | hansen-3 |   n-3   │
└─────┴──────────┴─────────┘
```


### 4. Nth

Set criteria (Select column: <u>name</u>, Number of the string: <u>1</u>, String separator: <u>-</u>, String mode: <u>Nth</u>)

```
┌─────┬──────────┬──────────┐
│ idx │ name     │ name_nth │
├─────┼──────────┼──────────┤
│  1  │ tom-1    │  tom     │
│  2  │ jerry-2  │  jerry   │
│  3  | hansen-3 |  hansen  │
└─────┴──────────┴──────────┘
```


### 5. Nmax

Set criteria (Select column: <u>name</u>, Number of the string: <u>2</u>, String separator: <u>-</u>, String mode: <u>Nmax</u>)

```
┌─────┬──────────┬────────────┬────────────┐
│ idx │ name     │ name_nmax1 │ name_nmax2 │
├─────┼──────────┼────────────┼────────────┤
│  1  │ tom-1    │  tom       │     1      │
│  2  │ jerry-2  │  jerry     │     2      │
│  3  | hansen-3 |  hansen    │     3      │
└─────┴──────────┴────────────┴────────────┘
```