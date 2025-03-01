<h1>InsightSQL</h1>

English | [中文](./README_CN.md)

> Rust Data Processing Toolbox, base on Tauri.

### screenshot

* command
 ![cmd.png](/demo/cmd.png)

* polars SQL for query

  ![sqlp.gif](/demo/sqlp.gif)

* CSV to xlsx

  ![csv2xlsx.gif](/demo/csv2xlsx.gif)

* Excel to CSV

  ![excel2csv.gif](/demo/excel2csv.gif)

* Cat

  ![cat.gif](/demo/cat.gif)

* Rename

  ![rename.gif](/demo/rename.gif)

## ✨Features

| Function | Description |
| ------- | ----------- |
| Sqlp | Execute Polars SQL queries against several files (Support Excel, CSV, Parquet) |
| Apply | Apply series of string, math transformations to given CSV column/s |
| Cat | Merge multiple CSV or Excel files into one CSV or xlsx file (Support Polars and CSV engine) |
| Excel  to CSV | Batch convert Excel to CSV (Support converting all sheets or specifying sheets) |
| Count | Count the rows of CSV files (Instantaneous with an index) |
| CSV to xlsx | Batch convert CSV to xlsx |
| Rename | Rename the columns of a CSV |
| Select | Select, re-order columns |
| Search | Match the corresponding row in a column (equal, contains, startswith, regex) |
| Fill | Fill empty fields in selected columns of a CSV |
| Split | Split one CSV file into many CSV files (by rows or by lines, uses multithreading to go faster if an index is present when splitting by rows) |
| Skip | Skip rows form CSV |
| Enumerate | Add a new column enumerating the lines of a CSV file |
| Chinese to Pinyin | Convert Chinese to Pinyin for specific column in CSV. |
| Replace | Replace CSV data using a regex |
| Join | Joins two sets of CSV data on the specified columns |
| Sort | Sorts CSV data lexicographically |
| Slice | Slicing of CSV column (like pandas str.slice and str.split('').str[n]) |


### 🍖How to use?

* Download [InsightSQL.7z](https://github.com/tansen87/sqlp/releases/), extract and run InsightSQL.exe


### 🏃‍Runtime Environment

* Node.js 18+
* pnpm 9.9.0+
* 🦀Rust 1.82.0+

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

### See also
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
* [qsv](https://github.com/jqnatividad/qsv)
