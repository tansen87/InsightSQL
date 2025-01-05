<h1>InsightSQL</h1>

English | [中文](./README_CN.md)

> A tool that can quickly view Excel, CSV and Parquet using polars SQL, base on Tauri.

### screenshot
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

* select

  ![select.gif](/demo/select.gif)


## ✨Features

- [x] Use Polars SQL for querying, allows to load Excel, CSV and Parquet files, support saving as csv, xlsx or paquet files
- [x] Merge multiple CSV or Excel files into one CSV or xlsx file (use Polars **concat_lf_diagonal**)
- [x] Batch convert Excel to CSV
- [x] Count the rows of CSV files
- [x] Batch convert CSV to Excel
- [x] Rename the columns of a CSV
- [x] Select, re-order columns
- [x] Select fields matching rows
- [x] Fill empty fields in selected columns of a CSV
- [x] Split one CSV file into many CSV files
- [x] Add an index for a CSV
- [x] Drop headers from CSVs
- [x] Convert Chinese to Pinyin for specific column in CSV
- [x] Replace CSV data using a regex
- [x] Apply series of string, math transformations to given CSV column/s


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

### Thanks
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
* [qsv](https://github.com/jqnatividad/qsv)
