<h1>sqlp</h1>
> A tool for quickly viewing CSV

### 截图

![image](/demo/screen1.png)

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




### 首页登录

* 账号: `admin`

* 密码: `admin123`



### 安装依赖

```sh
pnpm i
```

### 运行

```sh
pnpm tauri:dev
```

### 打包

```sh
pnpm tauri:build
```



### release

* [sqlp](https://github.com/tansen87/sqlp/releases/)

### video

* [bilibili](https://www.bilibili.com/video/BV1XS411c7zd/?spm_id_from=333.999.0.0&vd_source=5ee5270944c6e7a459e1311330bf455c)

### 致谢
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
