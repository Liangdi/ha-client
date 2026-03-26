/// 快速入门示例 - 最常用的功能
///
/// 运行方式: cargo run --example quick_start
///
/// 环境变量:
/// - HA_URL: Home Assistant URL
/// - HA_TOKEN: 长期访问令牌

use ha_client::HaClient;

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

    // 1. 查询状态
    let switch = client.entity("switch.ce_shi_usb_1");
    let state = switch.get_state().await?;
    println!("当前状态: {}", state);

    // 2. 判断并控制（使用 _and_wait 方法等待状态确认）
    if switch.is_off().await {
        println!("开关关闭，正在开启...");
        switch.turn_on_and_wait(5000).await?;
        println!("✓ 已开启");
    } else {
        println!("开关开启，正在关闭...");
        switch.turn_off_and_wait(5000).await?;
        println!("✓ 已关闭");
    }

    // 3. 打印信息（无需额外延迟，状态已经确认）
    switch.print_info().await?;

    Ok(())
}
