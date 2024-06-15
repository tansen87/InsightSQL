<h1>sqlp</h1>

> 截图

![image](https://github.com/tansen87/sqlp/assets/98570790/100967af-df90-4137-bd5d-32a3ee5dc5d1)

### 安装依赖

```sh
pnpm i
```

### 运行

```sh
# 桌面端
pnpm tauri:dev
```

### 打包

```sh
# 桌面端
pnpm tauri:build
```

### 图标生成
* 修改public文件夹下的app-icon.png图标为实际项目图标，格式为1024x1024px的png，然后执行下面的命令即可一键生成所有平台的icon并放在src-tauri/icons文件夹中
pnpm icon

### 致谢
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
