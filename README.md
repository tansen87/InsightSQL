<h1>InsightSQL</h1>

English | [中文](./README_CN.md)

> InsightSQL is a Tauri based data processing toolbox designed to simplify data operations and management. It provides an intuitive graphical user interface (GUI). Supports processing of multiple file formats, including Excel, CSV, Parquet, etc. Users can complete complex data processing tasks such as data queries, transformations, merges, joins, sorts, slices, format conversions, etc. through simple operations.

### ScreenShot

* command
 ![cmd.png](/docs/img/cmd.png)

* Polars SQL for query

  ![sqlp.gif](/docs/img/sqlp.gif)

* Rename

  ![rename.gif](/docs/img/rename.gif)

## ✨Features

| Function | Description |
| ------- | ----------- |
| Sqlp | Execute Polars SQL queries against several files (Support Excel, CSV, Parquet) |
| Apply | Apply series of string, math transformations to given CSV column/s |
| [Cat](./docs/cat.md) | Merge multiple CSV or Excel files into one CSV or xlsx file (Support Polars and CSV engine) |
| Excel  to CSV | Batch convert Excel to CSV (Support converting all sheets or specifying sheets) |
| Count | Count the rows of CSV files (Instantaneous with an index) |
| CSV to xlsx | Batch convert CSV to xlsx |
| Rename | Rename the columns of a CSV |
| Select | Select, re-order columns |
| [Search](./docs/search.md) | Match the corresponding row in a column (equal, contains, starts with, ends with, regex) |
| Fill | Fill empty fields in selected columns of a CSV |
| Split | Split one CSV file into many CSV files (by rows or by lines, uses multithreading to go faster if an index is present when splitting by rows) |
| Skip | Skip rows form CSV |
| Enumerate | Add a new column enumerating the lines of a CSV file |
| Chinese to Pinyin | Convert Chinese to Pinyin for specific column in CSV |
| Replace | Replace CSV data using a regex |
| Join | Joins two sets of CSV data on the specified columns |
| Sort | Sorts CSV data lexicographically |
| [Slice](./docs/slice.md) | Slicing of CSV column (like pandas str.slice and str.split('').str[n]) |
| Reverse | Reverse order of rows in a CSV |
| Transpose | Transpose rows/columns of a CSV |


### 🍖How to use?

* For more details, please refer to [release](https://github.com/tansen87/sqlp/releases/)


### 🏃‍Runtime Environment

* Node.js 20.19+
* pnpm 10.0+
* 🦀Rust 1.88.0+

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

* [bilibili](https://www.bilibili.com/video/BV1XS411c7zd/?spm_id_from=333.999.0.0&vd_source=5ee5270944c6e7a459e1311330bf455c) (The video hasn't been updated for a long time)

### See also
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
* [qsv](https://github.com/jqnatividad/qsv)
