<h1>InsightSQL</h1>

[English](./README.md) | 中文

> 一个可以使用SQL快速查看Excel、CSV和Parquet的工具，基于tauri。

### 截图
* Polars SQL 查询
![sqlp](/demo/sqlp.png)
* cat
![cat](/demo/cat.png)
* rename
![rename](/demo/rename.png)
* select
![select](/demo/select.png)



## ✨Features

- [x] 允许加载本地Excel、CSV和Parquet文件
- [x] 支持导出为CSV或xlsx文件
- [x] 使用Polars SQL进行查询
- [x] 合并多个CSV、Excel文件为1个CSV或xlsx文件
- [x] 批量将Excel转为CSV
- [x] 批量统计CSV文件的行数
- [x] 批量将CSV转为xlsx
- [x] 重命名CSV的表头
- [x] 选择、重新排序、复制或删除CSV列
- [x] 找出与特定条件匹配的字段 (equal、contains、startswith)
- [x] 填充CSV特定列中的空值
- [x] 将一个CSV按指定行数拆分为多个CSV


### 🍖如何使用?

* 下载[InsightSQL.7z](https://github.com/tansen87/sqlp/releases/)，提取并运行InsightSQL.exe

### 🧀两种查询模式

1. 直接输入SQL语句，例如:

   ```sql
   select * from `filename`
   ```

2. 编写一个SQL脚本(如下为SQL脚本)，然后选择它并运行它，例如:

   ```sql
   create table temp as select * from "filename" where code like '1%';
   select * from temp limit 10;
   ```

### 🏃‍运行环境

* Node.js 20+
* pnpm 9.9.0+
* 🦀Rust 1.81.0+

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

* [bilibili](https://www.bilibili.com/video/BV1XS411c7zd/?spm_id_from=333.999.0.0&vd_source=5ee5270944c6e7a459e1311330bf455c)

### 致谢
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
* [qsv](https://github.com/jqnatividad/qsv)
