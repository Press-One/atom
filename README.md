# atom

### docker-compose 运行 atom

#### 准备 `atom.env`

```
POSTGRES_PASSWORD=<YOUR-POSTGRES-PASSWORD>
POSTGRES_DB=atom
RUST_LOG=debug
RUST_BACKTRACE=full
DATABASE_URL=postgresql://postgres:<YOUR-POSTGRES-PASSWORD>@postgres/atom
EOS_BASE_URL=https://prs-bp-dev.press.one/api/chain
TOPIC=<YOUR-TOPIC-ADDRESS>;<YOUR-WEBHOOK-URL>
BIND_PORT=7070
BIND_ADDRESS=0.0.0.0:7070
ENCRYPTION_KEY=<YOUR-ENCRYPTION-KEY>
IV_PREFIX=<YOUR-IV-PREFIX>
THREAD_NUM=3
```

注：`POSTGRES_PASSWORD`、`DATABASE_URL`、`TOPIC`、`ENCRYPTION_KEY`、`IV_PREFIX` 需要根据自己的情况修改

#### build docker image

```
docker build -f Dockerfile -t atom
```

国内用户使用下面的命令：

```
docker build -f Dockerfile_cn -t atom
```

#### 运行

```
docker-compose up -d
```

注：第一次运行需要指定从哪个 `block_num` 开始抓取，修改 `docker-compose.yml`，在 `syncserver` 后增加 `block_num` 即可

## atom 开发

### 安装 rust/cargo

[请参考官方文档](https://rustlang-cn.org/office/rust/book/getting-started/ch01-01-installation.html)

### 安装依赖包

#### Ubuntu

`diesel_cli` 的依赖包：

```
sudo apt install libsqlite3-dev libmysqlclient-dev libpqxx-dev
```

`atom` 的依赖包：

```
sudo apt install -y libssl-dev
```

### 编译项目

debug 版本：

```
cargo build -j4
```

release 版本：

```
cargo build --release -j4
```

### 设置环境变量

需要先设置环境变量，程序会通过下面的环境变量获取 `配置信息`

请参考上面的 `.env`

也可以使用 [direnv](https://direnv.net/) 管理环境变量，使用方法自己参考官方文档。

### 更新数据库表结构

```
diesel migration run
```

### 同步链上数据

从 `last_status table` 中获取上次同步到哪个 `block_num` 了，然后用它作为起点，继续往后同步。

```
cargo run syncserver
```

从指定的 `block_num` 作为起点，继续往后同步。

```
cargo run syncserver <block_num>
```

比如，从 `block_num = 22270` 开始同步：

```
cargo run syncserver 22270
```

### 启动 web server

```
cargo run web
```
