<h1>sqlp</h1>

### 安装依赖

```sh
pnpm install
```

### 启动

```sh
# 桌面端
pnpm dev
```

```sh
# 浏览器端
pnpm browser:dev
```

### 打包

```sh
# 桌面端
pnpm build
```

```sh
# 浏览器端
pnpm browser:build
```

### 图标生成

```sh
# 修改public文件夹下的app-icon.png图标为实际项目图标，格式为1024x1024px的png，然后执行下面的命令即可一键生成所有平台的icon并放在src-tauri/icons文件夹中
pnpm icon
```