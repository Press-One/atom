# atom

### docker-compose 运行 atom

#### 准备配置文件

##### .env

```
RUST_LOG=info
RUST_BACKTRACE=full
ATOM_IMAGE=${your-atom-docker-image}
POSTGRES_PASSWORD=${db-password}
POSTGRES_DB=${db-name}
DATABASE_URL=postgresql://${db-user}:${db-password}@${db-host}/${db-name}
BIND_PORT=7070
```

注：

- `diesel` 不支持从 `Settings.toml` 获取配置文件，所以需要该配置文件
- `postgresql` 的默认 `${db-user}` 是 `postgres`

##### Settings.toml

配置文件名为：`Settings.toml`，**文件名必须是它，否则无法找到配置文件**

```
[atom]
db_url = "postgresql://<DB_USER>:<DB_PASSWORD>@<DB_HOST>/<DB_NAME>"  # 根据自己的数据库配置修改
# prs_base_url 支持 `http://a[1-2].com`，它将会展开为 `http://a1.com` 、`http://a2.com`；请求时随机选择其中之一
prs_base_url = "https://prs-bp[1-3].press.one/api/chain"
bind_address = "0.0.0.0:8080"  # web 服务监听地址
sentry_dsn = ""  # 可选，配置后可以在 sentry 上收到异常报警
xml_output_dir = "output"  # 生成 xml 文件的目录

# 配置 topic 信息，每个topic有自己的配置信息
[[topics]]
name = "your-topic-name"  # 可选，仅仅是作为备注信息
topic = "topic address"
webhook = "https://your-webhook-url"  # 可选，不配置就不会调用 webhook
# encryption_key, iv_prefix 用来解密链上数据；除非上链时没有加密，否则必填
encryption_key = "xxx"
iv_prefix = "yyy"

# 配置另一个 topic
[[topics]]
name = "yet-another-topic-name"
topic = "topic address"
webhook = "https://your-webhook-url"
encryption_key = "..."
iv_prefix = "..."

# 参考上面两个配置，可以配置更多 topic
```

#### build docker image

```
docker build -f Dockerfile -t pressone/atom
```

国内用户使用下面的命令：

```
docker build -f Dockerfile_cn -t pressone/atom
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

从 `last_status table` 中获取上次某个 `topic` 同步到哪个 `block_num` 了，然后用它作为起点，继续往后同步。

```
cargo run syncserver
```

注：默认从该 `topic` 的起始 `block number` 开始往后同步

### 启动 web server

```
cargo run web
```

## 请求 transaction

参数：

- topic
- blocknum 或 timestamp，作为起始点
- type, 比如：`PIP:2001`
- count，一次返回多少条数据

### 基于 blocknum 请求

```
http GET 'https://prs-bp-dev.press.one/api/chain/transactions?blocknum=682164&type=PIP:2001&count=10'
```

### 基于 timestamp 请求

```
http GET 'https://prs-bp-dev.press.one/api/chain/transactions?timestamp=2020-01-12T19:27:28.000Z&type=PIP:2001&count=2'
```

### 基于 topic 请求

```
http GET 'https://prs-bp1.press.one/api/chain/transactions?type=PIP:2001&topic=e6e2ff0ca504f7a2cf8237eac103f52f67fe5016&count=2'
```

注: 该接口也支持 `timestamp` 参数
