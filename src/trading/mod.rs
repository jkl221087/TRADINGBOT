use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::{CurrencyConfig, CurrencyStatus, TradingStatus, Position};
use crate::types::{OrderRequest, OrderType, OrderSide};
use crate::strategy::{MACDStrategy, TradingStrategy, MarketDepth, MarketTicker};
use crate::exchange::bingx::BingXClient;
use chrono::{Duration, Utc};
use serde_json;

pub struct TradingManager {
    client: Arc<BingXClient>,
    currencies: Arc<RwLock<HashMap<String, CurrencyStatus>>>,
    strategies: Arc<RwLock<HashMap<String, MACDStrategy>>>,
}

impl TradingManager {
    pub fn new(client: BingXClient) -> Self {
        Self {
            client: Arc::new(client),
            currencies: Arc::new(RwLock::new(HashMap::new())),
            strategies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // 获取客户端引用
    pub fn get_client(&self) -> &BingXClient {
        &self.client
    }

    // 添加新的交易币种
    pub async fn add_currency(&self, config: CurrencyConfig) {
        let mut currencies = self.currencies.write().await;
        let mut strategies = self.strategies.write().await;
        
        let status = CurrencyStatus {
            config: config.clone(),
            status: TradingStatus::Active,
            last_update: Utc::now().timestamp_millis(),
            current_position: None,
        };
        
        currencies.insert(config.symbol.clone(), status);
        strategies.insert(
            config.symbol.clone(),
            MACDStrategy::new(12, 26, 9)
        );
    }

    // 移除交易币种
    pub async fn remove_currency(&self, symbol: &str) {
        let mut currencies = self.currencies.write().await;
        let mut strategies = self.strategies.write().await;
        
        currencies.remove(symbol);
        strategies.remove(symbol);
    }

    // 获取币种状态
    pub async fn get_currency_status(&self, symbol: &str) -> Option<CurrencyStatus> {
        let currencies = self.currencies.read().await;
        currencies.get(symbol).cloned()
    }

    // 更新币种状态
    pub async fn update_currency_status(&self, symbol: &str, status: TradingStatus) {
        if let Some(currency) = self.currencies.write().await.get_mut(symbol) {
            currency.status = status;
            currency.last_update = Utc::now().timestamp_millis();
        }
    }

    // 获取所有币种状态
    pub async fn get_all_status(&self) -> Vec<(String, CurrencyStatus)> {
        let currencies = self.currencies.read().await;
        currencies.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    // 下单功能
    pub async fn place_order(&self, symbol: &str, side: OrderSide, price: f64) -> Result<(), Box<dyn std::error::Error>> {
        let currencies = self.currencies.read().await;
        let currency = currencies.get(symbol)
            .ok_or_else(|| format!("未找到币种配置: {}", symbol))?;

        if currency.status != TradingStatus::Active {
            return Err(format!("币种 {} 当前不可交易", symbol).into());
        }

        // 计算止盈止损价格
        let (take_profit_price, stop_loss_price) = match side {
            OrderSide::Buy => (
                price * 1.10,  // 买入时，止盈价格为入场价格+10%
                price * 0.95,  // 买入时，止损价格为入场价格-5%
            ),
            OrderSide::Sell => (
                price * 0.90,  // 卖出时，止盈价格为入场价格-10%
                price * 1.05,  // 卖出时，止损价格为入场价格+5%
            ),
        };

        // 构造止盈止损JSON
        let take_profit = serde_json::json!({
            "type": "TAKE_PROFIT_MARKET",
            "stopPrice": take_profit_price,
            "workingType": "MARK_PRICE",
            "closePosition": true
        }).to_string();

        let stop_loss = serde_json::json!({
            "type": "STOP_MARKET",
            "stopPrice": stop_loss_price,
            "workingType": "MARK_PRICE",
            "closePosition": true
        }).to_string();

        // 开仓订单
        let order = OrderRequest {
            symbol: symbol.to_string(),
            order_type: OrderType::Market,
            side: side.clone(),
            quantity: currency.config.min_qty,  // 使用配置中的最小数量
            timestamp: Utc::now().timestamp_millis(),
            stop_price: None,
            working_type: None,
            take_profit: Some(take_profit),
            stop_loss: Some(stop_loss),
        };

        match self.client.place_order(order).await {
            Ok(response) => {
                if response.code == 0 {
                    if let Some(data) = response.data {
                        println!("\n开仓成功:");
                        println!("订单ID: {}", data.order.order_id);
                        println!("交易对: {}", data.order.symbol);
                        println!("方向: {}", data.order.side);
                        println!("数量: {}", data.order.quantity);
                        println!("入场价格: {}", price);
                        println!("止盈价格: {:.2}", take_profit_price);
                        println!("止损价格: {:.2}", stop_loss_price);

                        println!("\n风险管理:");
                        println!("最大止损: {:.2}%", 5.0);
                        println!("预期盈利: {:.2}%", 10.0);
                        println!("盈亏比: 1:2");

                        // 更新币种状态
                        if let Some(currency) = self.currencies.write().await.get_mut(symbol) {
                            currency.current_position = Some(Position {
                                symbol: symbol.to_string(),
                                side: side.clone(),
                                quantity: data.order.quantity,
                                entry_price: price,
                                unrealized_pnl: 0.0,
                                leverage: currency.config.leverage,
                            });
                        }
                    }
                } else {
                    println!("开仓��败: {}", response.msg);
                }
            }
            Err(e) => println!("开仓错误: {}", e),
        }

        Ok(())
    }

    // 监控所有币种
    pub async fn monitor_all(&self) {
        loop {
            let currencies = self.currencies.read().await;
            let mut strategies = self.strategies.write().await;
            
            for (symbol, currency) in currencies.iter() {
                if currency.status != TradingStatus::Active {
                    continue;
                }
                
                // 获取市场数据
                let depth = match self.client.get_depth(&symbol, Some(20)).await {
                    Ok(depth_data) => {
                        let asks: Vec<(f64, f64)> = depth_data.asks.iter()
                            .filter_map(|ask| {
                                let price = ask[0].parse().ok()?;
                                let quantity = ask[1].parse().ok()?;
                                Some((price, quantity))
                            })
                            .collect();
                        let bids: Vec<(f64, f64)> = depth_data.bids.iter()
                            .filter_map(|bid| {
                                let price = bid[0].parse().ok()?;
                                let quantity = bid[1].parse().ok()?;
                                Some((price, quantity))
                            })
                            .collect();
                        Some(MarketDepth { asks, bids })
                    }
                    Err(e) => {
                        println!("{} - 获取深度数据失败: {}", symbol, e);
                        None
                    }
                };

                // 获取24小时行情
                let ticker = match self.client.get_ticker(Some(symbol)).await {
                    Ok(tickers) if !tickers.is_empty() => {
                        let t = &tickers[0];
                        Some(MarketTicker {
                            price_change_percent: t.price_change_percent.parse().unwrap_or_default(),
                            high_price: t.high_price.parse().unwrap_or_default(),
                            low_price: t.low_price.parse().unwrap_or_default(),
                            last_price: t.last_price.parse().unwrap_or_default(),
                            volume: t.volume.parse().unwrap_or_default(),
                            bid_price: t.bid_price.parse().unwrap_or_default(),
                            ask_price: t.ask_price.parse().unwrap_or_default(),
                        })
                    }
                    Ok(_) => None,
                    Err(e) => {
                        println!("{} - 获取24小时行情失败: {}", symbol, e);
                        None
                    }
                };

                // 获取K线数据
                if let Ok(mut klines) = self.client.get_klines(
                    symbol,
                    crate::types::Interval::FiveMinutes,
                    Some(Utc::now() - Duration::hours(2)),
                    Some(Utc::now()),
                    Some(24),
                ).await {
                    klines.reverse();
                    
                    if let Some(strategy) = strategies.get_mut(symbol) {
                        // 更新策略数据
                        for kline in &klines {
                            strategy.add_price(kline.close);
                        }
                        
                        // 获取最新K线数据
                        if let Some(latest_kline) = klines.last() {
                            println!("\n{} - 市场状况更新", symbol);
                            println!("最新价格: {:.2}", latest_kline.close);
                            
                            if let Some(t) = &ticker {
                                println!("24小时变动: {:.2}%", t.price_change_percent);
                                println!("��内波动率: {:.2}%", 
                                    (t.high_price - t.low_price) / t.low_price * 100.0);
                                let position = (t.last_price - t.low_price) / 
                                    (t.high_price - t.low_price) * 100.0;
                                println!("价格位置: 日内区间的 {:.1}%", position);
                            }
                            
                            // 检查交易信号
                            if strategy.should_buy(latest_kline.close, depth.as_ref(), ticker.as_ref()) {
                                println!("\n>>> {} - 发现买入信号!", symbol);
                                if let Err(e) = self.place_order(symbol, OrderSide::Buy, latest_kline.close).await {
                                    println!("下单失败: {}", e);
                                }
                            } else if strategy.should_sell(latest_kline.close, depth.as_ref(), ticker.as_ref()) {
                                println!("\n<<< {} - 发现卖出信号!", symbol);
                                if let Err(e) = self.place_order(symbol, OrderSide::Sell, latest_kline.close).await {
                                    println!("下单失败: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            
            // 等待一定时间后再次检查
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}