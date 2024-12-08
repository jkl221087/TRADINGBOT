use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Kline {
    pub open_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<(f64, f64)>,
    pub asks: Vec<(f64, f64)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub price: f64,
    pub quantity: f64,
    pub time: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Interval {
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "4h")]
    FourHours,
    #[serde(rename = "1d")]
    OneDay,
}

#[derive(Debug, Serialize, Clone)]
pub enum OrderType {
    #[serde(rename = "LIMIT")]
    Limit,
    #[serde(rename = "MARKET")]
    Market,
    #[serde(rename = "STOP_MARKET")]
    StopMarket,
    #[serde(rename = "TAKE_PROFIT_MARKET")]
    TakeProfitMarket,
}

#[derive(Debug, Serialize, Clone)]
pub enum OrderSide {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

#[derive(Debug, Serialize)]
pub enum PositionSide {
    #[serde(rename = "BOTH")]
    Both,
    #[serde(rename = "LONG")]
    Long,
    #[serde(rename = "SHORT")]
    Short,
}

#[derive(Debug, Serialize)]
pub struct OrderRequest {
    pub symbol: String,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: f64,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<OrderResponseData>,
}

#[derive(Debug, Deserialize)]
pub struct OrderResponseData {
    pub order: OrderData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
    #[serde(rename = "orderId")]
    pub order_id: i64,
    #[serde(rename = "orderID")]
    pub order_id_2: String,
    pub symbol: String,
    pub position_side: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub price: f64,
    pub quantity: f64,
    pub stop_price: f64,
    pub working_type: String,
    #[serde(rename = "clientOrderID")]
    pub client_order_id: String,
    pub time_in_force: String,
    pub price_rate: f64,
    pub stop_loss: String,
    pub take_profit: String,
    pub reduce_only: bool,
    pub activation_price: f64,
    pub close_position: String,
    pub stop_guaranteed: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MACD {
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone)]
pub struct CurrencyConfig {
    pub symbol: String,
    pub base_currency: String,    // 基础货币 (e.g., "BTC")
    pub quote_currency: String,   // 计价货币 (e.g., "USDT")
    pub min_qty: f64,            // 最小交易数量
    pub price_precision: u32,     // 价格精度
    pub qty_precision: u32,       // 数量精度
    pub min_notional: f64,       // 最小名义价值
    pub leverage: u32,           // 杠杆倍数
}

impl CurrencyConfig {
    pub fn new(
        symbol: &str,
        base_currency: &str,
        quote_currency: &str,
        min_qty: f64,
        price_precision: u32,
        qty_precision: u32,
        min_notional: f64,
        leverage: u32,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            base_currency: base_currency.to_string(),
            quote_currency: quote_currency.to_string(),
            min_qty,
            price_precision,
            qty_precision,
            min_notional,
            leverage,
        }
    }
}

// 交易状态
#[derive(Debug, Clone, PartialEq)]
pub enum TradingStatus {
    Active,
    Suspended,
    Error(String),
}

// 币种交易状态
#[derive(Debug, Clone)]
pub struct CurrencyStatus {
    pub config: CurrencyConfig,
    pub status: TradingStatus,
    pub last_update: i64,
    pub current_position: Option<Position>,
}

// 持仓信息
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub entry_price: f64,
    pub unrealized_pnl: f64,
    pub leverage: u32,
}