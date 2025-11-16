# Rustacos

一个用 Rust 编写的 Nacos 灵感来源的服务发现和配置管理系统。

## 功能特性

### 🚀 服务发现
- 服务实例注册与注销
- 健康检查与心跳机制
- 服务列表查询
- 支持集群和分组管理
- 实例权重管理

### ⚙️ 配置管理
- 配置发布与获取
- 多命名空间支持
- 配置版本管理
- 多种配置格式支持 (JSON, YAML, Properties)
- 配置监听与推送

### 💊 健康检查
- HTTP 健康检查
- TCP 健康检查
- 自定义健康检查
- 故障实例自动剔除

### 🌐 Web 管理界面
- 直观的服务管理界面
- 配置管理控制台
- 实时监控仪表板
- 响应式设计

### 💾 数据持久化
- 内存存储 (默认)
- SQLite 数据库支持
- 可扩展的存储接口

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

# 编译
cargo build --release

# 运行服务器
cargo run -- --port 8848
```

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

## 客户端示例

### Rust 客户端

运行示例客户端：

```bash
cargo run --example client_example
```

### HTTP 客户端

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
├── src/
│   ├── api/          # HTTP API 服务器和路由
│   ├── naming/       # 服务发现核心逻辑
│   ├── config/       # 配置管理核心逻辑
│   ├── health/       # 健康检查机制
│   ├── storage/      # 数据存储抽象层
│   ├── cli/          # 命令行参数处理
│   └── lib.rs        # 库入口
├── static/           # Web 界面静态文件
├── examples/         # 示例代码
└── Cargo.toml        # 项目配置
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