pub mod bingx;

#[allow(unused_imports)]
use crate::types::{OrderBook, Trade, Position};

#[allow(dead_code)]
pub trait Exchange {
    async fn get_price(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error>>;
    async fn place_order(&self, symbol: &str, side: OrderSide, quantity: f64, price: f64) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_balance(&self) -> Result<f64, Box<dyn std::error::Error>>;
}

#[allow(dead_code)]
pub enum OrderSide {
    Buy,
    Sell,
} 