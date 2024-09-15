<h1>InsightSQL</h1>

> A tool that can quickly view Excel, CSV and Parquet using SQL

### screenshot
* polars SQL for query
![sqlp](/demo/sqlp.png)
* cat
![cat](/demo/cat.png)
* rename
![rename](/demo/rename.png)
* select
![select](/demo/select.png)



## ✨Features

- [x] Allows to load local Excel, CSV and Parquet files
- [x] Support exporting as csv or xlsx files
- [x] Use Polars SQL for querying
- [x] Concatenate CSV and Excel files
- [x] Exports Excel to a csv file
- [x] Count the rows of CSV files
- [x] Exports csv to a xlsx file
- [x] Rename the columns of a CSV
- [x] Select, re-order, duplicate or drop columns


### 🍖How to use?

* Download [InsightSQL.7z](https://github.com/tansen87/sqlp/releases/), extract and run InsightSQL.exe

### 🧀Two query modes

1. Directly enter SQL statements, such as:

   ```sql
   select * from `filename`
   ```

2. Write an SQL script, then select it and run it, for example:

   ```sql
   create table temp as select * from "filename" where code like '1%';
   select * from temp limit 10;
   ```

### 🏃‍Runtime Environment

* Node.js 20+
* pnpm 9.9.0+
* 🦀Rust 1.81.0+

## 🚀Development

1. Clone this repositories

   ```bash
   git clone https://github.com/tansen87/InsightSQL.git
   ```

2. cd to the directory of the project

   ```bash
   cd InsightSQL
   ```

3. Install dependencies

   ```bash
   pnpm i
   ```

4. Development

   ```bash
   pnpm tauri:dev
   ```

5. Build

   ```bash
   pnpm tauri:build
   ```

### video

* [bilibili](https://www.bilibili.com/video/BV1XS411c7zd/?spm_id_from=333.999.0.0&vd_source=5ee5270944c6e7a459e1311330bf455c)

### Thanks
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
* [qsv](https://github.com/jqnatividad/qsv)
