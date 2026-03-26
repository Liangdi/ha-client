/// 完整的 API 使用示例
///
/// 运行方式: cargo run --example usage
///
/// 环境变量:
/// - HA_URL: Home Assistant URL
/// - HA_TOKEN: 长期访问令牌

use ha_client::HaClient;
use ha_client::serde_json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // ==================== 配置 ====================
    let ha_url = std::env::var("HA_URL")
        .expect("请设置环境变量 HA_URL");
    let ha_token = std::env::var("HA_TOKEN")
        .expect("请设置环境变量 HA_TOKEN");

    // 创建客户端（只需一次，自动管理连接信息）
    let client = HaClient::new(&ha_url, &ha_token);

    // ==================== 基础使用 ====================

    // 1. 获取配置信息
    println!("=== 系统信息 ===");
    let config = client.get_config().await?;
    println!("版本: {}", config.version);
    println!("位置: {}", config.location_name);
    println!("时区: {}\n", config.time_zone);

    // 2. 获取所有实体
    println!("=== 所有实体 ===");
    let all_entities = client.get_all_entities().await?;
    println!("共有 {} 个实体\n", all_entities.len());

    // 3. 创建实体控制器
    let switch1 = client.entity("switch.ce_shi_usb_1");

    // ==================== 实体操作 ====================

    // 方式1: 基本操作
    println!("=== 方式1: 基本操作 ===");

    // 获取状态
    let state = switch1.get_state().await?;
    println!("当前状态: {}", state);

    // 判断状态
    if switch1.is_on().await {
        println!("开关是开启的");
    } else if switch1.is_off().await {
        println!("开关是关闭的");
    }

    // 控制开关
    switch1.turn_on().await?;   // 开启
    switch1.turn_off().await?;  // 关闭
    switch1.toggle().await?;    // 切换

    // 方式2: 打印完整信息
    println!("\n=== 方式2: 完整信息 ===");
    switch1.print_info().await?;

    // 方式3: 智能控制（根据当前状态自动切换）
    println!("\n=== 方式3: 智能控制 ===");
    let current = switch1.get_state().await?;
    if current == "off" {
        println!("开关关闭，正在开启...");
        switch1.turn_on().await?;
    } else if current == "on" {
        println!("开关开启，正在关闭...");
        switch1.turn_off().await?;
    }

    // ==================== 高级用法 ====================

    // 1. 通用服务调用
    println!("\n=== 高级用法 ===");

    // 例如: light 域的 turn_on 服务可以传递额外参数
    // 注意：如果没有 light 实体，这会失败，我们用 try 块处理
    let light = client.entity("light.living_room");
    match light.call_service(
        "light",
        "turn_on",
        serde_json::json!({
            "brightness_pct": 50,  // 亮度 50%
            "color_temp": 400,      // 色温
        })
    ).await {
        Ok(_) => println!("✓ 灯光控制成功"),
        Err(e) => println!("⚠ 灯光控制失败（可能不存在）: {}", e),
    }

    // 2. 批量控制多个实体
    println!("\n=== 批量控制 ===");
    let switches = vec![
        client.entity("switch.ce_shi_usb_1"),
        client.entity("switch.ce_shi_usb_2"),
        client.entity("switch.ce_shi_socket_1"),
    ];

    // 批量开启
    println!("批量开启...");
    for sw in &switches {
        match sw.turn_on().await {
            Ok(_) => println!("✓ {} 已开启", sw.entity_id()),
            Err(e) => println!("✗ {} 开启失败: {}", sw.entity_id(), e),
        }
    }

    // 等待
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 批量关闭
    println!("\n批量关闭...");
    for sw in &switches {
        match sw.turn_off().await {
            Ok(_) => println!("✓ {} 已关闭", sw.entity_id()),
            Err(e) => println!("✗ {} 关闭失败: {}", sw.entity_id(), e),
        }
    }

    // 3. 监控状态变化
    println!("\n=== 状态监控 ===");
    for i in 1..=3 {
        let state = switch1.get_state().await?;
        println!("[{}] 状态: {}", i, state);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // ==================== 错误处理 ====================

    println!("\n=== 错误处理示例 ===");

    // 不存在的实体
    let invalid = client.entity("switch.nonexistent");
    match invalid.get_state().await {
        Ok(state) => println!("状态: {}", state),
        Err(e) => println!("✗ 查询失败（预期错误）: {}", e),
    }

    // 失败的操作
    match invalid.turn_on().await {
        Ok(_) => println!("操作成功"),
        Err(e) => println!("✗ 操作失败（预期错误）: {}", e),
    }

    // ==================== 查找特定实体 ====================

    println!("\n=== 查找所有开关实体 ===");
    let all_entities = client.get_all_entities().await?;
    let switch_entities: Vec<_> = all_entities
        .iter()
        .filter(|e| e.entity_id.as_ref().unwrap_or(&"".to_string()).starts_with("switch."))
        .collect();

    println!("找到 {} 个开关实体:", switch_entities.len());
    for (i, entity) in switch_entities.iter().take(10).enumerate() {
        let friendly_name = entity.attributes.as_ref()
            .and_then(|a| a.friendly_name.as_deref())
            .unwrap_or("N/A");
        println!("  {}. {} - {} (状态: {})",
            i + 1,
            entity.entity_id.as_ref().unwrap(),
            friendly_name,
            entity.state
        );
    }

    Ok(())
}
