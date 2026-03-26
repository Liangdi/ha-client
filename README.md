# ha-client

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Cargo](https://img.shields.io/badge/crates.io-v0.1.0--blue)](https://crates.io/crates/ha-client)

一个简单且类型安全的 Rust Home Assistant API 客户端库。

## ✨ 特性

- 🚀 **简单易用** - 直观的 API 设计，快速上手
- 🔒 **类型安全** - 利用 Rust 类型系统确保 API 调用安全
- ⚡ **异步支持** - 基于 Tokio 的完整异步 I/O
- 🎯 **实体控制** - 支持开关、灯光等实体的状态查询和控制
- ⏱️ **状态等待** - 内置状态确认和超时机制
- 📦 **零成本抽象** - 基于 `homeassistant-rs` 的轻量级封装

## 📦 安装

在 `Cargo.toml` 中添加以下依赖：

```toml
[dependencies]
ha-client = "0.1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## 🚀 快速开始

### 基础用法

```rust
use ha_client::HaClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建客户端
    let client = HaClient::new(
        "http://localhost:8123",
        "your-long-lived-access-token"
    );

    // 获取实体控制器
    let switch = client.entity("switch.example");

    // 开启开关
    switch.turn_on().await?;

    // 查询状态
    let state = switch.get_state().await?;
    println!("当前状态: {}", state);

    Ok(())
}
```

### 使用环境变量（推荐）

创建 `.env` 文件：

```env
HA_URL=http://localhost:8123
HA_TOKEN=your-long-lived-access-token
```

代码示例：

```rust
use ha_client::HaClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    let ha_url = std::env::var("HA_URL")?;
    let ha_token = std::env::var("HA_TOKEN")?;

    let client = HaClient::new(&ha_url, &ha_token);
    let switch = client.entity("switch.example");

    // 智能控制：根据当前状态自动切换
    if switch.is_off().await {
        switch.turn_on().await?;
    } else {
        switch.turn_off().await?;
    }

    Ok(())
}
```

## 📚 主要功能

### 1. 实体控制

```rust
let switch = client.entity("switch.living_room");

// 基本操作
switch.turn_on().await?;      // 开启
switch.turn_off().await?;     // 关闭
switch.toggle().await?;       // 切换状态

// 等待状态确认（带超时）
switch.turn_on_and_wait(5000).await?;   // 开启并等待状态确认
switch.turn_off_and_wait(5000).await?;  // 关闭并等待状态确认
```

### 2. 状态查询

```rust
let sensor = client.entity("sensor.temperature");

// 获取状态值
let state = sensor.get_state().await?;
println!("温度: {}", state);

// 判断状态
if sensor.is_on().await {
    println!("传感器在线");
}

// 等待状态变化
let success = sensor.wait_for_state("on", 10000).await?;
println!("状态变更结果: {}", success);
```

### 3. 打印实体信息

```rust
let light = client.entity("light.bedroom");
light.print_info().await?;
```

输出示例：
```
实体信息:
  ID: light.bedroom
  友好名称: 卧室灯
  状态: on
  最后更新: 2026-03-26T10:30:45.123456+00:00
```

### 4. 通用服务调用

```rust
use ha_client::serde_json;

let light = client.entity("light.living_room");

// 调用灯光服务，传递额外参数
light.call_service(
    "light",
    "turn_on",
    serde_json::json!({
        "brightness_pct": 50,  // 亮度 50%
        "color_temp": 400,     // 色温
        "rgb_color": [255, 0, 0]  // 红色
    })
).await?;
```

### 5. 批量控制

```rust
let switches = vec![
    client.entity("switch.socket1"),
    client.entity("switch.socket2"),
    client.entity("switch.socket3"),
];

// 批量开启
for sw in &switches {
    sw.turn_on().await?;
}

// 等待 2 秒
tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

// 批量关闭
for sw in &switches {
    sw.turn_off().await?;
}
```

### 6. 系统信息

```rust
// 获取配置信息
let config = client.get_config().await?;
println!("Home Assistant 版本: {}", config.version);
println!("位置: {}", config.location_name);
println!("时区: {}", config.time_zone);

// 获取所有实体
let all_entities = client.get_all_entities().await?;
println!("共有 {} 个实体", all_entities.len());

// 查找特定类型的实体
let switch_entities: Vec<_> = all_entities
    .iter()
    .filter(|e| e.entity_id.as_ref()
        .unwrap_or(&"".to_string())
        .starts_with("switch."))
    .collect();

println!("找到 {} 个开关实体", switch_entities.len());
```

### 7. 错误处理

```rust
let invalid = client.entity("switch.nonexistent");

match invalid.get_state().await {
    Ok(state) => println!("状态: {}", state),
    Err(e) => println!("查询失败: {}", e),
}

match invalid.turn_on().await {
    Ok(_) => println!("操作成功"),
    Err(e) => println!("操作失败: {}", e),
}
```

## 🔧 配置说明

### 获取 Home Assistant 访问令牌

1. 登录 Home Assistant
2. 点击用户头像 → 滚动到底部 → **创建令牌**
3. 输入令牌名称（如 "ha-client"）
4. 复制生成的令牌（只显示一次，请妥善保存）

### URL 格式

- 本地: `http://localhost:8123` 或 `http://192.168.1.100:8123`
- 远程: `https://your-home-assistant.duckdns.org`

## 📖 示例代码

本项目提供了完整的示例代码：

```bash
# 运行快速入门示例
cargo run --example quick_start

# 运行完整用法示例
cargo run --example usage
```

**注意**：运行示例前需要先配置 `.env` 文件：

```env
HA_URL=http://localhost:8123
HA_TOKEN=your-token-here
```

## 🎯 API 概览

### HaClient

| 方法 | 说明 |
|------|------|
| `new(url, token)` | 创建新客户端 |
| `entity(id)` | 获取实体控制器 |
| `get_config()` | 获取系统配置 |
| `get_all_entities()` | 获取所有实体 |

### HaEntity

| 方法 | 说明 |
|------|------|
| `entity_id()` | 获取实体 ID |
| `get_state()` | 获取当前状态值 |
| `get_full_state()` | 获取完整状态信息 |
| `is_on()` | 判断是否开启 |
| `is_off()` | 判断是否关闭 |
| `turn_on()` | 开启实体 |
| `turn_off()` | 关闭实体 |
| `toggle()` | 切换状态 |
| `turn_on_and_wait(ms)` | 开启并等待确认 |
| `turn_off_and_wait(ms)` | 关闭并等待确认 |
| `wait_for_state(state, ms)` | 等待状态变化 |
| `call_service(domain, service, data)` | 通用服务调用 |
| `print_info()` | 打印实体信息 |

## 🛠️ 开发

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo clippy
```

### 格式化代码

```bash
cargo fmt
```

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📮 联系方式

- 作者: Liangdi <wu@liangdi.me>
- 仓库: https://github.com/Liangdi/ha-client

## 🔗 相关资源

- [Home Assistant 官方文档](https://www.home-assistant.io/docs/)
- [homeassistant-rs](https://crates.io/crates/homeassistant-rs) - 本库的底层依赖

---

**注意**: 请妥善保管您的访问令牌，不要将其提交到版本控制系统。建议将 `.env` 文件添加到 `.gitignore` 中。
