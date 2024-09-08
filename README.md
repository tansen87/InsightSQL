<h1>sqlp</h1>
> A tool that can quickly view Excel, CSV, and Parquet using SQL

### 截图

![image](/demo/screen.png)



### 如何使用?

1. 下载[release](https://github.com/tansen87/sqlp/releases/)，解压运行sqlp.exe
2. Open File => 打开需要查询的文件(xls, xlsx, xlsm, xlsb, csv, parquet)，csv文件需要选择Open File旁边的分割符
3. 编写sql语句，点击Execute开始查询



### 两种查询模式

1. 查询较简单时，在Execute上面的方框内输入sql语句，比如

   ```sql
   select * from `filename`
   ```

2. 查询较复杂时，在Execute上面的方框内填写sql脚本的地址，比如`E:/Desktop/test_data/sqlp_test/test.sql`，然后编写sql语句(注意用`;`分隔sql语句)

   ```sql
   create table temp as select * from "GL" where code like '1%';
   select * from temp limit 10;
   ```



### 运行环境

* Node.js 18+
* pnpm 8.x+
* rust 1.80.1+

### 源码安装

1. 克隆该项目

   ```bash
   git clone https://github.com/tansen87/sqlp.git
   ```

2. cd到该项目的目录

3. 安装依赖

   ```bash
   pnpm i
   ```

4. 运行

   ```bash
   pnpm tauri:dev
   ```

5. 打包

   ```bash
   pnpm tauri:build
   ```

### video

* [bilibili](https://www.bilibili.com/video/BV1XS411c7zd/?spm_id_from=333.999.0.0&vd_source=5ee5270944c6e7a459e1311330bf455c)

### 致谢
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
