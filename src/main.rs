mod config;
mod exchange;
mod strategy;
mod types;
mod trading;

use chrono::{Duration, TimeZone, Utc};
use dotenv::dotenv;
use exchange::bingx::BingXClient;
use types::{Interval, OrderType, OrderSide, OrderRequest, CurrencyConfig};
use trading::TradingManager;
use std::env;
use tokio::time::{sleep, Duration as TokioDuration};
use serde_json;

async fn init_currencies() -> Vec<CurrencyConfig> {
    vec![
        CurrencyConfig::new(
            "BTC-USDT",
            "BTC",
            "USDT",
            1.0,    // 最小数量
            1,        // 价格精度
            3,        // 数量精度
            5.0,      // 最小名义价值
            20,       // 杠杆倍数
        ),
        CurrencyConfig::new(
            "ETH-USDT",
            "ETH",
            "USDT",
            30.0,     // 最小数量
            2,        // 价格精度
            3,        // 数量精度
            5.0,      // 最小名义价值
            20,       // 杠杆倍数
        ),
        CurrencyConfig::new(
            "SOL-USDT",
            "SOL",
            "USDT",
            410.0,      // 最小数量
            3,        // 价格精度
            1,        // 数量精度
            5.0,      // 最小名义价值
            20,       // 杠杆倍数
        ),
        CurrencyConfig::new(
            "XRP-USDT",
            "XRP",
            "USDT",
            10000.0,    // 最小数量 (调整为100个XRP，考虑到XRP价格较低)
            4,        // 价格精度
            1,        // 数量精度
            5.0,      // 最小名义价值
            20,       // 杠杆倍数
        ),
        CurrencyConfig::new(
            "BNB-USDT",
            "BNB",
            "USDT",
            16.0,    // 最小数量 (调整为100个XRP，考虑到XRP价格较低)
            4,        // 价格精度
            1,        // 数量精度
            5.0,      // 最小名义价值
            20,       // 杠杆倍数
        ),
        CurrencyConfig::new(
            "1000PEPE-USDT",
            "1000PEPE",
            "USDT",
            980000.0,    // 最小数量 (调整为100个XRP，考虑到XRP价格较低)
            4,        // 价格精度
            1,        // 数量精度
            5.0,      // 最小名义价值
            20,       // 杠杆倍数
        ),
        CurrencyConfig::new(
            "SUI-USDT",
            "SUI",
            "USDT",
            5800.0,    // 最小数量 (调整为100个XRP，考虑到XRP价格较低)
            4,        // 价格精度
            1,        // 数量精度
            5.0,      // 最小名义价值
            20,       // 杠杆倍数
        ),
        CurrencyConfig::new(
            "ARB-USDT",
            "ARB",
            "USDT",
            38000.0,    // 最小数量 (调整为100个XRP，考虑到XRP价格较低)
            4,        // 价格精度
            1,        // 数量精度
            5.0,      // 最小名义价值
            20,       // 杠杆倍数
        ),
    ]
}

async fn print_currency_status(manager: &TradingManager) {
    let all_status = manager.get_all_status().await;
    
    if all_status.is_empty() {
        println!("当前没有配置任何交易币种");
        return;
    }
    
    for (symbol, status) in all_status {
        println!("\n币种状态 - {}:", symbol);
        println!("交易状态: {:?}", status.status);
        println!("最后更新: {}", 
            Utc.timestamp_millis_opt(status.last_update)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
        );
        
        // 显示配置信息
        println!("\n配置信息:");
        println!("  最小交易数量: {}", status.config.min_qty);
        println!("  价格精度: {}", status.config.price_precision);
        println!("  数量精度: {}", status.config.qty_precision);
        println!("  最小名义价值: {}", status.config.min_notional);
        println!("  杠杆倍数: {}", status.config.leverage);
        
        // 显示持仓信息
        if let Some(position) = status.current_position {
            println!("\n当前持仓:");
            println!("  方向: {:?}", position.side);
            println!("  数量: {}", position.quantity);
            println!("  入场价格: {}", position.entry_price);
            println!("  未实现盈亏: {:.2}%", position.unrealized_pnl);
            println!("  杠杆倍数: {}", position.leverage);
        } else {
            println!("\n当前无持仓");
        }
    }
}

async fn init_manager() -> TradingManager {
    let api_key = env::var("BINGX_API_KEY").expect("未设置 BINGX_API_KEY");
    let api_secret = env::var("BINGX_API_SECRET").expect("未设置 BINGX_API_SECRET");
    
    let client = BingXClient::new(api_key, api_secret);
    TradingManager::new(client)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("加密货币交易机器人启动中...");
    
    let manager = init_manager().await;
    
    loop {
        println!("\n请选择操作:");
        println!("1. 初始化预设币种");
        println!("2. 查看所有币种状态");
        println!("3. 添加新币种");
        println!("4. 暂停币种交易");
        println!("5. 恢复币种交易");
        println!("6. 测试买入订单");
        println!("7. 测试卖出订单");
        println!("8. 查看市场��度");
        println!("9. 查看24小时行情");
        println!("10. 开始监控交易");
        println!("0. 退出程序");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("读取输入失败");
        
        match input.trim() {
            "1" => {
                println!("初始化币种配置...");
                let currencies = init_currencies().await;
                
                for currency in currencies {
                    println!("添加币种: {}", currency.symbol);
                    manager.add_currency(currency).await;
                }
                println!("预设币种初始化完成!");
            }
            "2" => {
                println!("获取所有币种状态...");
                print_currency_status(&manager).await;
            }
            "3" => {
                println!("请输入币种信息 (格式: 交易对,基础币,计价币,最小数量,价格精度,数量精度,最小名义价值,杠杆倍数)");
                println!("例如: BTC-USDT,BTC,USDT,0.001,1,3,5.0,20");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("读取输入失败");
                
                let parts: Vec<&str> = input.trim().split(',').collect();
                if parts.len() == 8 {
                    let config = CurrencyConfig::new(
                        parts[0],
                        parts[1],
                        parts[2],
                        parts[3].parse().unwrap_or(0.001),
                        parts[4].parse().unwrap_or(1),
                        parts[5].parse().unwrap_or(3),
                        parts[6].parse().unwrap_or(5.0),
                        parts[7].parse().unwrap_or(20),
                    );
                    
                    manager.add_currency(config).await;
                    println!("币种添加成功!");
                } else {
                    println!("输入格式错误!");
                }
            }
            "4" => {
                println!("请输入要暂停的币种交易对 (例如: BTC-USDT):");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("读取输入失败");
                
                manager.update_currency_status(
                    input.trim(),
                    types::TradingStatus::Suspended
                ).await;
                println!("币种交易已暂停!");
            }
            "5" => {
                println!("请输入要恢复的币种交易�� (例如: BTC-USDT):");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("读取输入失败");
                
                manager.update_currency_status(
                    input.trim(),
                    types::TradingStatus::Active
                ).await;
                println!("币种交易已恢复!");
            }
            "6" | "7" => {
                println!("请输入交易对 (例如: BTC-USDT):");
                let mut symbol = String::new();
                std::io::stdin().read_line(&mut symbol).expect("读取输入失败");
                let symbol = symbol.trim();

                let side = if input.trim() == "6" {
                    println!("测试买入订单...");
                    OrderSide::Buy
                } else {
                    println!("测试卖出订单...");
                    OrderSide::Sell
                };

                // 获取当前价格
                match manager.get_client().get_latest_price(symbol).await {
                    Ok(price) => {
                        println!("当前价格: {}", price);
                        if let Err(e) = manager.place_order(symbol, side, price).await {
                            println!("下单失败: {}", e);
                        }
                    }
                    Err(e) => println!("获取价格失败: {}", e),
                }
            }
            "8" => {
                println!("请输入交易对 (例如: BTC-USDT):");
                let mut symbol = String::new();
                std::io::stdin().read_line(&mut symbol).expect("读取输入失败");
                
                println!("获取市场深度信息...");
                match manager.get_client().print_depth_info(symbol.trim(), Some(20)).await {
                    Ok(_) => println!("\n深度信息获取成功"),
                    Err(e) => println!("获取深度信息失败: {}", e),
                }
            }
            "9" => {
                println!("请输入交易对 (例如: BTC-USDT):");
                let mut symbol = String::new();
                std::io::stdin().read_line(&mut symbol).expect("读取输入失败");
                
                println!("获取24小时行情信息...");
                match manager.get_client().print_ticker_info(Some(symbol.trim())).await {
                    Ok(_) => println!("\n24小时行情获取成功"),
                    Err(e) => println!("获取24小时行情失败: {}", e),
                }
            }
            "10" => {
                println!("开���监控所有币种...");
                manager.monitor_all().await;
            }
            "0" => {
                println!("程序退出!");
                break;
            }
            _ => println!("无效的选择"),
        }
    }
}

// ... rest of the code stays the same ...