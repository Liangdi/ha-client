//! # Home Assistant 客户端库
//!
//! 这是一个用于与 Home Assistant 实例交互的 Rust 客户端库。
//!
//! ## 特性
//!
//! - 简单易用的 API
//! - 支持实体控制（开关、切换等）
//! - 支持状态查询和等待
//! - 完整的异步支持
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use ha_client::HaClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = HaClient::new("http://localhost:8123", "your-token");
//!     let switch = client.entity("switch.example");
//!
//!     switch.turn_on().await?;
//!     Ok(())
//! }
//! ```

use homeassistant_rs::{hass, structs::{StatesResponse, ConfigResponse}};

// Re-export serde_json for user convenience
pub use homeassistant_rs::serde_json;

/// Home Assistant 客户端
///
/// 用于与 Home Assistant 实例通信的主客户端。
#[derive(Debug, Clone)]
pub struct HaClient {
    ha_url: String,
    ha_token: String,
}

impl HaClient {
    /// 创建新的客户端
    ///
    /// # 参数
    ///
    /// * `ha_url` - Home Assistant 实例的 URL（如 "http://localhost:8123"）
    /// * `ha_token` - 长期访问令牌
    ///
    /// # 示例
    ///
    /// ```rust
    /// use ha_client::HaClient;
    ///
    /// let client = HaClient::new("http://localhost:8123", "your-token");
    /// ```
    pub fn new(ha_url: &str, ha_token: &str) -> Self {
        Self {
            ha_url: ha_url.to_string(),
            ha_token: ha_token.to_string(),
        }
    }

    /// 获取实体控制器
    ///
    /// # 参数
    ///
    /// * `entity_id` - 实体 ID（如 "switch.example"）
    ///
    /// # 返回
    ///
    /// 返回一个 [`HaEntity`] 实例，用于控制该实体
    pub fn entity(&self, entity_id: &str) -> HaEntity {
        HaEntity::new(entity_id, &self.ha_url, &self.ha_token)
    }

    /// 获取所有实体
    ///
    /// # 返回
    ///
    /// 返回所有实体的状态列表
    pub async fn get_all_entities(&self) -> anyhow::Result<Vec<StatesResponse>> {
        hass().states(
            Some(self.ha_url.clone()),
            Some(self.ha_token.clone()),
            None,
        ).await
    }

    /// 获取 Home Assistant 配置信息
    ///
    /// # 返回
    ///
    /// 返回包含版本、位置、时区等配置信息的 [`ConfigResponse`]
    pub async fn get_config(&self) -> anyhow::Result<ConfigResponse> {
        hass().config(
            Some(self.ha_url.clone()),
            Some(self.ha_token.clone()),
        ).await
    }
}

/// Home Assistant 实体控制器
///
/// 用于控制单个实体的状态和行为。
#[derive(Debug, Clone)]
pub struct HaEntity {
    entity_id: String,
    ha_url: String,
    ha_token: String,
}

impl HaEntity {
    /// 创建一个新的实体控制器
    fn new(entity_id: &str, ha_url: &str, ha_token: &str) -> Self {
        Self {
            entity_id: entity_id.to_string(),
            ha_url: ha_url.to_string(),
            ha_token: ha_token.to_string(),
        }
    }

    /// 获取实体 ID
    ///
    /// # 返回
    ///
    /// 返回实体 ID 字符串（如 "switch.example"）
    pub fn entity_id(&self) -> &str {
        &self.entity_id
    }

    /// 获取当前状态
    ///
    /// # 返回
    ///
    /// 返回实体的当前状态值（如 "on", "off"）
    pub async fn get_state(&self) -> anyhow::Result<String> {
        let states = hass().states(
            Some(self.ha_url.clone()),
            Some(self.ha_token.clone()),
            Some(&self.entity_id),
        ).await?;

        states
            .first()
            .map(|s| s.state.clone())
            .ok_or_else(|| anyhow::anyhow!("未找到实体: {}", self.entity_id))
    }

    /// 获取完整状态信息
    ///
    /// # 返回
    ///
    /// 返回包含状态、属性等完整信息的 [`StatesResponse`]
    pub async fn get_full_state(&self) -> anyhow::Result<StatesResponse> {
        let states = hass().states(
            Some(self.ha_url.clone()),
            Some(self.ha_token.clone()),
            Some(&self.entity_id),
        ).await?;

        states
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("未找到实体: {}", self.entity_id))
    }

    /// 判断是否开启
    ///
    /// # 返回
    ///
    /// 如果状态为 "on" 返回 `true`，否则返回 `false`
    pub async fn is_on(&self) -> bool {
        matches!(self.get_state().await.as_deref(), Ok("on"))
    }

    /// 判断是否关闭
    ///
    /// # 返回
    ///
    /// 如果状态为 "off" 返回 `true`，否则返回 `false`
    pub async fn is_off(&self) -> bool {
        matches!(self.get_state().await.as_deref(), Ok("off"))
    }

    /// 等待状态变为指定值（带超时）
    ///
    /// # 参数
    ///
    /// * `expected_state` - 期望的状态值（如 "on", "off"）
    /// * `timeout_ms` - 超时时间（毫秒），默认 5000ms
    ///
    /// # 返回
    ///
    /// * `Ok(true)` - 状态已变为期望值
    /// * `Ok(false)` - 超时仍未达到期望状态
    pub async fn wait_for_state(&self, expected_state: &str, timeout_ms: u64) -> anyhow::Result<bool> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let check_interval = std::time::Duration::from_millis(100);

        loop {
            if let Ok(state) = self.get_state().await {
                if state == expected_state {
                    return Ok(true);
                }
            }

            if start.elapsed() >= timeout {
                return Ok(false);
            }

            tokio::time::sleep(check_interval).await;
        }
    }

    /// 开启实体
    ///
    /// # 返回
    ///
    /// 返回 API 响应的 JSON 值
    pub async fn turn_on(&self) -> anyhow::Result<serde_json::Value> {
        let domain = self.get_domain();
        let payload = serde_json::json!({ "entity_id": self.entity_id });

        hass().request().service(
            Some(self.ha_url.clone()),
            Some(self.ha_token.clone()),
            &domain,
            "turn_on",
            payload,
            false,
        ).await
    }

    /// 开启实体并等待状态确认
    ///
    /// # 参数
    ///
    /// * `timeout_ms` - 超时时间（毫秒）
    pub async fn turn_on_and_wait(&self, timeout_ms: u64) -> anyhow::Result<()> {
        self.turn_on().await?;
        self.wait_for_state("on", timeout_ms).await?;
        Ok(())
    }

    /// 关闭实体
    ///
    /// # 返回
    ///
    /// 返回 API 响应的 JSON 值
    pub async fn turn_off(&self) -> anyhow::Result<serde_json::Value> {
        let domain = self.get_domain();
        let payload = serde_json::json!({ "entity_id": self.entity_id });

        hass().request().service(
            Some(self.ha_url.clone()),
            Some(self.ha_token.clone()),
            &domain,
            "turn_off",
            payload,
            false,
        ).await
    }

    /// 关闭实体并等待状态确认
    ///
    /// # 参数
    ///
    /// * `timeout_ms` - 超时时间（毫秒）
    pub async fn turn_off_and_wait(&self, timeout_ms: u64) -> anyhow::Result<()> {
        self.turn_off().await?;
        self.wait_for_state("off", timeout_ms).await?;
        Ok(())
    }

    /// 切换状态（如果开启则关闭，如果关闭则开启）
    ///
    /// # 返回
    ///
    /// 返回 API 响应的 JSON 值
    pub async fn toggle(&self) -> anyhow::Result<serde_json::Value> {
        let domain = self.get_domain();
        let payload = serde_json::json!({ "entity_id": self.entity_id });

        hass().request().service(
            Some(self.ha_url.clone()),
            Some(self.ha_token.clone()),
            &domain,
            "toggle",
            payload,
            false,
        ).await
    }

    /// 调用服务（通用方法）
    ///
    /// # 参数
    ///
    /// * `domain` - 服务域（如 "light", "switch"）
    /// * `service` - 服务名（如 "turn_on", "turn_off"）
    /// * `data` - 服务数据（JSON 对象）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// # use ha_client::HaClient;
    /// # use ha_client::serde_json;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let client = HaClient::new("http://localhost:8123", "token");
    /// let light = client.entity("light.living_room");
    /// light.call_service(
    ///     "light",
    ///     "turn_on",
    ///     serde_json::json!({"brightness_pct": 50}),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        data: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let mut payload = serde_json::json!({ "entity_id": self.entity_id });
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                payload[key] = value.clone();
            }
        }

        hass().request().service(
            Some(self.ha_url.clone()),
            Some(self.ha_token.clone()),
            domain,
            service,
            payload,
            false,
        ).await
    }

    /// 获取实体域（从 entity_id 中提取）
    ///
    /// 例如: "switch.ce_shi_usb_1" -> "switch"
    fn get_domain(&self) -> String {
        self.entity_id
            .split('.')
            .next()
            .unwrap_or("homeassistant")
            .to_string()
    }

    /// 打印实体信息
    ///
    /// 打印实体的详细信息，包括 ID、友好名称、状态和最后更新时间
    pub async fn print_info(&self) -> anyhow::Result<()> {
        let state = self.get_full_state().await?;
        let friendly_name = state
            .attributes
            .as_ref()
            .and_then(|a| a.friendly_name.as_deref())
            .unwrap_or("N/A");

        println!("实体信息:");
        println!("  ID: {}", self.entity_id);
        println!("  友好名称: {}", friendly_name);
        println!("  状态: {}", state.state);
        println!("  最后更新: {:?}", state.last_updated);

        Ok(())
    }
}
