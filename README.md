# Rustacos

一个用 Rust 编写的 Nacos 灵感来源的服务发现和配置管理系统。

## 功能特性

### 🚀 服务发现
- 服务实例注册/注销/心跳
- 心跳 TTL 自动标记不健康（内置调度器）
- 服务与实例列表查询
- 分组与集群字段
- 实例权重
- 实例变更 SSE 推送（topic=instance）

### ⚙️ 配置管理
- 配置发布/获取/删除
- 多命名空间
- 配置历史与回滚
- 导入/导出
- 多种配置格式 (JSON/YAML/Properties/HTML/TEXT)
- 配置变更 SSE 推送（topic=config）
- 前端支持历史 vs 历史并排 Diff、历史 vs 当前 Diff

### 💊 健康
- 心跳机制
- TTL 定时清理并标记 unhealthy

### 🌐 Web 管理界面
- 直观的服务管理界面
- 配置管理控制台
- 实时监控仪表板
- 响应式设计

### 💾 数据持久化
- 内存存储 (默认，DashMap)
- 端口/适配层设计，可扩展数据库/消息组件（后续适配）

## 快速开始

### 安装依赖

确保你的系统已安装 Rust 1.70+：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 编译运行

```bash
# 克隆项目
git clone <repository-url>
cd rustacos

# 直接运行（默认端口 8848，基于 app-bootstrap 装配）
cargo run

# 可指定端口
cargo run -- -p 8848

# 并行联调入口（可选，端口 8850）
cargo run --bin nextapp
```

### SSE 事件流

- 端点：`/nacos/v1/events/stream?topic=config|instance`
- 用途：
  - `topic=config`：配置变更事件（包含 namespace/group/data_id）
  - `topic=instance`：实例变更事件（包含 service_name）
- 前端已内置自动订阅，收到事件后自动刷新对应列表；也可自行通过 EventSource 订阅：

```javascript
const es = new EventSource('/nacos/v1/events/stream?topic=config');
es.onmessage = (e) => console.log('config event', e.data);
```

### 环境变量

- `SSE_AUTH_REQUIRED`：是否要求 SSE 订阅提供授权（Authorization 头或 `access_token` 查询参数）。默认开启（1/true）。关闭可设为 `0` 或 `false`。
- `HEARTBEAT_TTL_SECS`：实例最后心跳超过该秒数则标记 unhealthy。默认 `30`。
- `HEARTBEAT_SWEEP_SECS`：心跳扫描周期。默认 `10`。

### 命令行参数

```bash
rustacos [OPTIONS]

OPTIONS:
    -p, --port <PORT>          设置服务器端口 [default: 8848]
    -s, --storage <STORAGE>    存储类型 [default: memory] [possible values: memory, sqlite]
    -d, --db-path <DB_PATH>    SQLite 数据库路径 [default: data/rustacos.db]
```

## API 文档

### 服务发现 API

#### 注册实例
```http
POST /nacos/v1/ns/instance
Content-Type: application/json

{
  "ip": "127.0.0.1",
  "port": 8080,
  "service_name": "example-service",
  "group_name": "DEFAULT_GROUP",
  "cluster_name": "DEFAULT",
  "weight": 1.0,
  "ephemeral": true
}
```

#### 注销实例
```http
DELETE /nacos/v1/ns/instance/{service_name}/{instance_id}
```

#### 发送心跳
```http
POST /nacos/v1/ns/instance/beat
Content-Type: application/json

{
  "service_name": "example-service",
  "instance_id": "instance-uuid"
}
```

#### 获取实例列表
```http
GET /nacos/v1/ns/instance/list?service_name=example-service&cluster_name=DEFAULT
```

#### 获取服务列表
```http
GET /nacos/v1/ns/service/list
```

### 配置管理 API

#### 发布配置
```http
POST /nacos/v1/cs/configs
Content-Type: application/json

{
  "data_id": "example-config",
  "group": "DEFAULT_GROUP",
  "content": "app.name=example",
  "namespace": "public",
  "config_type": "properties"
}
```

#### 获取配置
```http
GET /nacos/v1/cs/configs?data_id=example-config&group=DEFAULT_GROUP&namespace=public
```

#### 删除配置
```http
DELETE /nacos/v1/cs/configs?data_id=example-config&group=DEFAULT_GROUP&namespace=public
```

#### 配置历史
```http
GET /nacos/v1/cs/configs/history?data_id=example-config&group=DEFAULT_GROUP&namespace=public
```

#### 历史回滚
```http
POST /nacos/v1/cs/configs/history/rollback
Content-Type: application/json

{
  "data_id": "example-config",
  "group": "DEFAULT_GROUP",
  "namespace": "public",
  "version": 1700000000
}
```

#### 导出配置
```http
GET /nacos/v1/cs/configs/export?namespace=public
```

#### 导入配置
```http
POST /nacos/v1/cs/configs/import
Content-Type: application/json

[{
  "data_id": "application.json",
  "group": "DEFAULT_GROUP",
  "namespace": "public",
  "content": "{ \"k\": \"v\" }",
  "format": "json"
}]
```

### 命名空间 API

#### 创建命名空间
```http
POST /nacos/v1/console/namespaces
Content-Type: application/json

{
  "namespace": "dev",
  "namespace_show_name": "开发环境",
  "namespace_desc": "开发环境配置"
}
```

#### 获取命名空间列表
```http
GET /nacos/v1/console/namespaces
```

## 客户端示例（HTTP）
 
使用 curl 注册服务：

```bash
curl -X POST http://localhost:8848/nacos/v1/ns/instance \
  -H "Content-Type: application/json" \
  -d '{
    "ip": "127.0.0.1",
    "port": 8080,
    "service_name": "test-service",
    "group_name": "DEFAULT_GROUP"
  }'
```

发送心跳：

```bash
curl -X POST http://localhost:8848/nacos/v1/ns/instance/beat \
  -H "Content-Type: application/json" \
  -d '{
    "service_name": "test-service",
    "instance_id": "instance-id"
  }'
```

订阅配置变更（SSE）：

```bash
curl -N http://localhost:8848/nacos/v1/events/stream?topic=config
```

## Web 管理界面

启动服务器后，访问以下地址打开管理界面：

```
http://localhost:8848
```

管理界面提供：
- 🏠 仪表板：系统概览和统计信息
- 📋 服务管理：服务注册、实例查看
- ⚙️ 配置管理：配置发布、编辑、删除
- 🗂️ 命名空间：环境隔离管理

## 架构设计

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Console   │    │   HTTP API      │    │   Client SDK    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
┌─────────────────────────────────┼─────────────────────────────────┐
│                    Rustacos Server                              │
├─────────────────────────────────┼─────────────────────────────────┤
│  Naming Service  │  Config Manager  │  Health Checker  │  Storage  │
└─────────────────────────────────┼─────────────────────────────────┘
                                 │
                    ┌─────────────────┴─────────────────┐
                    │         Storage Layer            │
                    ├─────────────────┬─────────────────┤
                    │    Memory       │    SQLite       │
                    └─────────────────┴─────────────────┘
```

## 开发

### 项目结构

```
rustacos/
├── crates/
│   ├── core-model/               # 领域模型（Config/Instance/Namespace/History）
│   ├── core-ports/               # 端口接口（Store/Notifier/Scheduler）
│   ├── core-usecase/             # 用例（发布/回滚等）
│   ├── adapters-storage-memory/  # 内存存储实现（DashMap）
│   ├── adapters-notify-sse/      # SSE 推送适配器（服务端广播）
│   ├── api-compat-nacos/         # Nacos 兼容 API 路由（Axum）
│   └── app-bootstrap/            # 应用装配与静态服务
├── src/
│   ├── bin/
│   │   ├── rustacos.rs           # 主入口（8848，使用 app-bootstrap）
│   │   └── nextapp.rs            # 联调入口（8850，可选）
│   └── frontend/                 # Leptos 前端（WASM）
├── static/                       # 前端静态资源（index.html、editor.js 等）
└── Cargo.toml                    # Workspace 配置
```

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo clippy
cargo fmt
```

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 致谢

本项目灵感来源于 [Nacos](https://github.com/alibaba/nacos)，致力于用 Rust 提供一个高性能、安全可靠的服务发现和配置管理解决方案。