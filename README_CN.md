<h1>InsightSQL</h1>

[English](./README.md) | 中文

> 基于Tauri的数据处理工具箱

### 截图

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
| Sqlp | 对多个文件执行Polars SQL查询 (支持Excel, CSV, Parquet) |
| Apply | 对给定的CSV列应用一系列字符串和数学转换 |
| Cat | 将多个CSV或Excel文件合并为一个CSV或xlsx文件 (支持Polars和CSV引擎) |
| Excel  to CSV | 批量将Excel转换为CSV (支持转换所有工作表或指定工作表) |
| Count | 统计CSV文件的行数 (带索引的瞬时值) |
| CSV to xlsx | 批量将CSV转换为xlsx |
| Rename | 重命名CSV的列 |
| Select | 选择、重新排序CSV的列 |
| Search | 匹配列中的相应行 (包含模式: equal, contains, startswith, regex) |
| Fill | 在CSV的选定列中填写空白字段 |
| Split | 将一个CSV文件拆分为多个CSV文件 (按rows或按lines,如果按行拆分时存在索引,则使用多线程来加快速度) |
| Skip | 跳过CSV中的行 |
| Enumerate | 添加一个新列,枚举CSV文件的行 |
| Chinese to Pinyin | 将CSV中特定列的中文转换为拼音 |
| Replace | 使用正则表达式替换CSV数据 |
| Join | 在指定列上连接两组CSV数据 |
| Sort | 对CSV排序 |
| Slice | CSV列的切片 (如pandas.str.slice和str.split('').str[n]) |

### 🍖如何使用?

* 详细可查看[release](https://github.com/tansen87/sqlp/releases/)


### 🏃‍运行环境

* Node.js 18+
* pnpm 9.9.0+
* 🦀Rust 1.85.0+

## 🚀开发

1. 克隆该仓库

   ```bash
   git clone https://github.com/tansen87/InsightSQL.git
   ```

2. cd到该项目的路径

   ```bash
   cd InsightSQL
   ```

3. 安装依赖

   ```bash
   pnpm i
   ```

4. 开发

   ```bash
   pnpm tauri:dev
   ```

5. 打包

   ```bash
   pnpm tauri:build
   ```

### 演示视频

* [bilibili](https://www.bilibili.com/video/BV1XS411c7zd/?spm_id_from=333.999.0.0&vd_source=5ee5270944c6e7a459e1311330bf455c) (视频很久未更新了)

### 致谢
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
* [qsv](https://github.com/jqnatividad/qsv)
