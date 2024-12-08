use crate::types::{
    Interval, Kline, OrderRequest, OrderResponse,
    OrderSide
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::BTreeMap;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Url;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

const API_BASE_URL: &str = "https://open-api-vst.bingx.com";

pub struct BingXClient {
    client: Client,
    api_secret: String,
}

#[derive(Debug, Deserialize)]
struct KlineResponse {
    code: i32,
    msg: String,
    data: Vec<KlineData>,
}

#[derive(Debug, Deserialize)]
struct KlineData {
    open: String,
    close: String,
    high: String,
    low: String,
    volume: String,
    time: i64,
}

#[derive(Debug, Deserialize)]
pub struct DepthData {
    #[serde(rename = "T")]
    pub timestamp: i64,
    pub asks: Vec<[String; 2]>,
    pub bids: Vec<[String; 2]>,
    #[serde(rename = "asksCoin")]
    pub asks_coin: Vec<[String; 2]>,
    #[serde(rename = "bidsCoin")]
    pub bids_coin: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
struct DepthResponse {
    code: i32,
    msg: String,
    data: Option<DepthData>,
}

#[derive(Debug, Deserialize)]
pub struct TickerData {
    pub symbol: String,
    #[serde(rename = "priceChange")]
    pub price_change: String,
    #[serde(rename = "priceChangePercent")]
    pub price_change_percent: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "lastQty")]
    pub last_qty: String,
    #[serde(rename = "highPrice")]
    pub high_price: String,
    #[serde(rename = "lowPrice")]
    pub low_price: String,
    pub volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
    #[serde(rename = "openPrice")]
    pub open_price: String,
    #[serde(rename = "openTime")]
    pub open_time: i64,
    #[serde(rename = "closeTime")]
    pub close_time: i64,
    #[serde(rename = "bidPrice")]
    pub bid_price: String,
    #[serde(rename = "bidQty")]
    pub bid_qty: String,
    #[serde(rename = "askPrice")]
    pub ask_price: String,
    #[serde(rename = "askQty")]
    pub ask_qty: String,
}

#[derive(Debug, Deserialize)]
struct TickerResponse {
    code: i32,
    msg: String,
    data: TickerResponseData,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TickerResponseData {
    Single(TickerData),
    Multiple(Vec<TickerData>),
}

impl BingXClient {
    pub fn new(api_key: String, api_secret: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("X-BX-APIKEY", HeaderValue::from_str(&api_key).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self {
            client,
            api_secret,
        }
    }

    fn sign(&self, params: &mut BTreeMap<String, String>) -> String {
        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&");

        let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: Interval,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>, Box<dyn Error>> {
        let mut url = format!(
            "{}/openApi/swap/v3/quote/klines?symbol={}&interval={}",
            API_BASE_URL,
            symbol,
            serde_json::to_string(&interval)?.replace("\"", "")
        );

        if let Some(start) = start_time {
            url.push_str(&format!("&startTime={}", start.timestamp_millis()));
        }
        if let Some(end) = end_time {
            url.push_str(&format!("&endTime={}", end.timestamp_millis()));
        }
        if let Some(limit_val) = limit {
            url.push_str(&format!("&limit={}", limit_val));
        }

        url.push_str(&format!("&timestamp={}", Utc::now().timestamp_millis()));

        println!("请求URL: {}", url);

        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        println!("API响应: {}", response_text);

        let kline_response: KlineResponse = serde_json::from_str(&response_text)?;

        if kline_response.code != 0 {
            return Err(format!("API错误: {}", kline_response.msg).into());
        }

        let klines = kline_response
            .data
            .into_iter()
            .map(|k| Kline {
                open_time: k.time,
                close_time: k.time,  // BingX API 只提供了一个时间戳
                open: k.open.parse().unwrap_or_default(),
                high: k.high.parse().unwrap_or_default(),
                low: k.low.parse().unwrap_or_default(),
                close: k.close.parse().unwrap_or_default(),
                volume: k.volume.parse().unwrap_or_default(),
            })
            .collect();

        Ok(klines)
    }

    pub async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, Box<dyn Error>> {
        // 构造基本参数
        let timestamp = Utc::now().timestamp_millis();
        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), order.symbol.clone());
        params.insert("side".to_string(), match order.side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        });
        params.insert("positionSide".to_string(), "LONG".to_string());
        params.insert("type".to_string(), "MARKET".to_string());
        params.insert("quantity".to_string(), format!("{}", order.quantity));
        params.insert("timestamp".to_string(), timestamp.to_string());
        params.insert("recvWindow".to_string(), "5000".to_string());

        // 添加止盈止损
        if let Some(take_profit) = order.take_profit {
            params.insert("takeProfit".to_string(), take_profit);
        }
        if let Some(stop_loss) = order.stop_loss {
            params.insert("stopLoss".to_string(), stop_loss);
        }

        // 生成签名字符串
        let param_str = {
            let mut keys: Vec<&String> = params.keys().collect();
            keys.sort();
            let encoded_params: Vec<String> = keys.iter()
                .map(|k| {
                    let value = params.get(*k).unwrap();
                    format!("{}={}", k, value)
                })
                .collect();
            encoded_params.join("&")
        };

        println!("签名前的字符串: {}", param_str);

        // 计算签名
        let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(param_str.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        // 构造URL并编码参数
        let mut url = Url::parse(&format!("{}/openApi/swap/v2/trade/order", API_BASE_URL))?;
        
        // 添加编码后的参数
        for (key, value) in &params {
            url.query_pairs_mut().append_pair(key, value);
        }
        
        // 添加签名
        url.query_pairs_mut().append_pair("signature", &signature);

        println!("\n下单请求URL: {}", url.as_str());
        println!("签名: {}", signature);

        // 发送请求
        let response = self.client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await?;

        let response_text = response.text().await?;
        println!("API响应: {}", response_text);

        let order_response: OrderResponse = serde_json::from_str(&response_text)?;
        
        if order_response.code != 0 {
            println!("\n下单结果:");
            println!("响应代码: {}", order_response.code);
            println!("响应消息: {}", order_response.msg);
            return Err(order_response.msg.into());
        }

        Ok(order_response)
    }

    pub async fn get_latest_price(&self, symbol: &str) -> Result<f64, Box<dyn Error>> {
        let url = format!(
            "{}/openApi/swap/v1/ticker/price?symbol={}&timestamp={}",
            API_BASE_URL,
            symbol,
            Utc::now().timestamp_millis()
        );

        println!("获取价格URL: {}", url);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        println!("价格响应: {}", response_text);

        #[derive(Debug, Deserialize)]
        struct PriceResponse {
            code: i32,
            msg: String,
            data: Option<PriceData>,
        }

        #[derive(Debug, Deserialize)]
        struct PriceData {
            symbol: String,
            price: String,
            #[serde(rename = "time")]
            timestamp: i64,
        }

        let price_response: PriceResponse = serde_json::from_str(&response_text)?;
        if price_response.code != 0 {
            return Err(format!("API错误: {}", price_response.msg).into());
        }

        if let Some(price_data) = price_response.data {
            Ok(price_data.price.parse()?)
        } else {
            Err("无价格数据".into())
        }
    }

    pub async fn get_depth(&self, symbol: &str, limit: Option<u32>) -> Result<DepthData, Box<dyn Error>> {
        let mut url = format!(
            "{}/openApi/swap/v2/quote/depth?symbol={}&timestamp={}",
            API_BASE_URL,
            symbol,
            Utc::now().timestamp_millis()
        );

        if let Some(limit_val) = limit {
            url.push_str(&format!("&limit={}", limit_val));
        }

        println!("获取深度信息URL: {}", url);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        println!("深度信息响应: {}", response_text);

        let depth_response: DepthResponse = serde_json::from_str(&response_text)?;
        if depth_response.code != 0 {
            return Err(format!("API错误: {}", depth_response.msg).into());
        }

        depth_response.data.ok_or_else(|| "无深度数据".into())
    }

    pub async fn print_depth_info(&self, symbol: &str, limit: Option<u32>) -> Result<(), Box<dyn Error>> {
        let depth = self.get_depth(symbol, limit).await?;
        
        println!("\n深度信息 - {}:", symbol);
        println!("时间戳: {}", depth.timestamp);
        
        println!("\n卖单 (价格/数量):");
        for (i, ask) in depth.asks.iter().enumerate().take(5) {
            println!("  {}: {} / {}", i+1, ask[0], ask[1]);
        }
        
        println!("\n买单 (价格/数量):");
        for (i, bid) in depth.bids.iter().enumerate().take(5) {
            println!("  {}: {} / {}", i+1, bid[0], bid[1]);
        }
        
        println!("\n卖单 (币数量):");
        for (i, ask) in depth.asks_coin.iter().enumerate().take(5) {
            println!("  {}: {} / {}", i+1, ask[0], ask[1]);
        }
        
        println!("\n买单 (币数量):");
        for (i, bid) in depth.bids_coin.iter().enumerate().take(5) {
            println!("  {}: {} / {}", i+1, bid[0], bid[1]);
        }

        Ok(())
    }

    pub async fn get_ticker(&self, symbol: Option<&str>) -> Result<Vec<TickerData>, Box<dyn Error>> {
        let mut url = format!(
            "{}/openApi/swap/v2/quote/ticker?timestamp={}",
            API_BASE_URL,
            Utc::now().timestamp_millis()
        );

        if let Some(sym) = symbol {
            url.push_str(&format!("&symbol={}", sym));
        }

        println!("获取24小时行情URL: {}", url);
        let response = self.client.get(&url).send().await?;
        let response_text = response.text().await?;
        println!("24小时行情响应: {}", response_text);

        let ticker_response: TickerResponse = serde_json::from_str(&response_text)?;
        if ticker_response.code != 0 {
            return Err(format!("API错误: {}", ticker_response.msg).into());
        }

        Ok(match ticker_response.data {
            TickerResponseData::Single(ticker) => vec![ticker],
            TickerResponseData::Multiple(tickers) => tickers,
        })
    }

    pub async fn print_ticker_info(&self, symbol: Option<&str>) -> Result<(), Box<dyn Error>> {
        let tickers = self.get_ticker(symbol).await?;
        
        for ticker in tickers {
            println!("\n24小时行情 - {}:", ticker.symbol);
            println!("价格变动: {} ({:.2}%)", 
                ticker.price_change,
                ticker.price_change_percent.parse::<f64>().unwrap_or_default()
            );
            println!("最新价格: {} (数量: {})", ticker.last_price, ticker.last_qty);
            println!("24小时最高: {}", ticker.high_price);
            println!("24小时最低: {}", ticker.low_price);
            println!("24小时成交量: {}", ticker.volume);
            println!("24小时成交额: {} USDT", ticker.quote_volume);
            println!("买一价格: {} (数量: {})", ticker.bid_price, ticker.bid_qty);
            println!("卖一价格: {} (数量: {})", ticker.ask_price, ticker.ask_qty);
            
            // 计算买卖价差
            let bid = ticker.bid_price.parse::<f64>().unwrap_or_default();
            let ask = ticker.ask_price.parse::<f64>().unwrap_or_default();
            let spread = (ask - bid) / bid * 100.0;
            println!("买卖价差: {:.3}%", spread);
            
            // 计算当前价格在日内区间的位置
            let high = ticker.high_price.parse::<f64>().unwrap_or_default();
            let low = ticker.low_price.parse::<f64>().unwrap_or_default();
            let current = ticker.last_price.parse::<f64>().unwrap_or_default();
            if high > low {
                let position = (current - low) / (high - low) * 100.0;
                println!("价格位置: 日内区间的 {:.1}%", position);
            }

            // 计算成交量分析
            let quote_volume = ticker.quote_volume.parse::<f64>().unwrap_or_default();
            let volume = ticker.volume.parse::<f64>().unwrap_or_default();
            let avg_price = if volume > 0.0 {
                quote_volume / volume
            } else {
                0.0
            };
            println!("平均成交价: {:.2} USDT", avg_price);
            
            // 计算价格波动率
            let high_low_range = (high - low) / low * 100.0;
            println!("日内波动率: {:.2}%", high_low_range);
        }
        
        Ok(())
    }
} 