/// 列出所有实体示例
///
/// 运行方式: cargo run --example list_entities
///
/// 环境变量:
/// - HA_URL: Home Assistant URL
/// - HA_TOKEN: 长期访问令牌

use ha_client::HaClient;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 配置
    let ha_url = std::env::var("HA_URL")
        .expect("请设置环境变量 HA_URL");
    let ha_token = std::env::var("HA_TOKEN")
        .expect("请设置环境变量 HA_TOKEN");

    // 创建客户端
    let client = HaClient::new(&ha_url, &ha_token);

    // 获取系统信息
    println!("=== Home Assistant 系统信息 ===");
    let config = client.get_config().await?;
    println!("版本: {}", config.version);
    println!("位置: {}", config.location_name);
    println!("时区: {}", config.time_zone);
    println!();

    // 获取所有实体
    let all_entities = client.get_all_entities().await?;

    println!("=== 实体统计 ===");
    println!("总实体数: {}", all_entities.len());
    println!();

    // 按域分组
    let mut by_domain: HashMap<String, Vec<&homeassistant_rs::structs::StatesResponse>> = HashMap::new();
    for entity in &all_entities {
        if let Some(entity_id) = &entity.entity_id {
            let domain = entity_id.split('.').next().unwrap_or("unknown");
            by_domain.entry(domain.to_string()).or_default().push(entity);
        }
    }

    println!("=== 按域分类 ===");
    let mut domains: Vec<_> = by_domain.keys().collect();
    domains.sort();

    for domain in domains {
        let count = by_domain[domain].len();
        println!("{:20} : {} 个", domain, count);
    }
    println!();

    // 列出所有实体（按字母排序）
    println!("=== 所有实体列表 ===");
    let mut sorted_entities: Vec<_> = all_entities.iter().collect();
    sorted_entities.sort_by(|a, b| {
        a.entity_id.as_ref().unwrap_or(&"".to_string())
            .cmp(b.entity_id.as_ref().unwrap_or(&"".to_string()))
    });

    for (i, entity) in sorted_entities.iter().enumerate() {
        let entity_id = entity.entity_id.as_ref().map(|s| s.as_str()).unwrap_or("?");
        let friendly_name = entity.attributes.as_ref()
            .and_then(|a| a.friendly_name.as_deref())
            .unwrap_or("N/A");

        println!("{:4}. [{:30}] {} - (状态: {})",
            i + 1,
            entity_id,
            friendly_name,
            entity.state
        );
    }

    println!();

    // 显示特定域的实体详情
    println!("=== 开关实体详情 ===");
    if let Some(switches) = by_domain.get("switch") {
        for (i, entity) in switches.iter().enumerate() {
            let entity_id = entity.entity_id.as_ref().unwrap();
            let friendly_name = entity.attributes.as_ref()
                .and_then(|a| a.friendly_name.as_deref())
                .unwrap_or("N/A");

            println!("{}. {}", i + 1, entity_id);
            println!("   友好名称: {}", friendly_name);
            println!("   状态: {}", entity.state);

            // 显示额外属性
            if let Some(last_changed) = &entity.last_changed {
                println!("   最后改变: {}", last_changed);
            }
            if let Some(last_updated) = &entity.last_updated {
                println!("   最后更新: {}", last_updated);
            }
            println!();
        }
    } else {
        println!("没有找到开关实体");
    }

    // 显示灯光实体
    println!("=== 灯光实体详情 ===");
    if let Some(lights) = by_domain.get("light") {
        for (i, entity) in lights.iter().enumerate() {
            let entity_id = entity.entity_id.as_ref().unwrap();
            let friendly_name = entity.attributes.as_ref()
                .and_then(|a| a.friendly_name.as_deref())
                .unwrap_or("N/A");

            println!("{}. {} - {} (状态: {})",
                i + 1,
                entity_id,
                friendly_name,
                entity.state
            );
        }
    } else {
        println!("没有找到灯光实体");
    }

    Ok(())
}
