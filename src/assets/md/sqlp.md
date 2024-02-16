# sql script

### 1.concat

```sql
SELECT * FROM table1
UNION ALL BY NAME
SELECT * FROM table2
```

### 2.select

```sql
SELECT * FROM table WHERE col = '0'

SELECT * FROM `All candidates2` LIMIT 10

SELECT * FROM table WHERE col like 'query%'

SELECT idx,name FROM table WHERE col like 'query%'
```

### 3.join

```sql
SELECT * FROM `All candidates` as all1
JOIN `All candidates2` as all2
ON all1.`CAND_ID` = all2.`CAND_ID`

SELECT * FROM `All candidates` as all1
LEFT JOIN `All candidates2` as all2
ON all1.`CAND_ID` = all2.`CAND_ID`
```

### 4.sum

```sql
SELECT SUM(cast(idx AS float)) FROM `All candidates2`
```

### 5.group by

```sql
SELECT 
CAND_ID,SUM(CAST(idx as float))
FROM `All candidates2`
GROUP BY CAND_ID
```

### 6.count

```sql
SELECT COUNT(*) FROM `All candidates2`
```

### 7.order by

```sql
SELECT 
CAND_ID,SUM(CAST(idx as float))
FROM `All candidates2`
GROUP BY CAND_ID
ORDER BY CAND_ID

SELECT 
CAND_ID,SUM(CAST(idx as float))
FROM `All candidates2`
GROUP BY CAND_ID
ORDER BY CAND_ID
DESC
```
