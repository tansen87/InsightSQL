<h1>InsightSQL</h1>

[English](./README.md) | 中文

> 一个可以使用polars SQL快速查看Excel、CSV和Parquet的工具，基于tauri。

### 截图
* polars SQL for query

  ![sqlp.gif](/demo/sqlp.gif)

* Cat

  ![cat.gif](/demo/cat.gif)

* Rename

  ![rename.gif](/demo/rename.gif)

* select

  ![select.gif](/demo/select.gif)


## ✨Features

- [x] 使用Polars SQL进行查询,支持Excel、CSV和parquet,可以保存为CSV, xlsx或parquet文件
- [x] 合并多个CSV、Excel文件为1个CSV或xlsx文件
- [x] 批量将Excel转为CSV
- [x] 批量统计CSV文件的行数
- [x] 批量将CSV转为xlsx
- [x] 重命名CSV的表头
- [x] 选择、重新排序CSV列
- [x] 找出与特定条件匹配的字段 (equal、contains、startswith)
- [x] 填充CSV特定列中的空值
- [x] 将一个CSV按指定行数拆分为多个CSV
- [x] 为CSV添加索引
- [x] 批量删除CSV的表头
- [x] 将CSV中特定列的中文转为拼音


### 🍖如何使用?

* 下载[InsightSQL.7z](https://github.com/tansen87/sqlp/releases/)，提取并运行InsightSQL.exe


### 🏃‍运行环境

* Node.js 18+
* pnpm 9.9.0+
* 🦀Rust 1.82.0+

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
