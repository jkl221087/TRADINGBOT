use crate::types::MACD;

// 定义市场深度数据结构
#[derive(Debug, Clone)]
pub struct MarketDepth {
    pub asks: Vec<(f64, f64)>,  // (价格, 数量)
    pub bids: Vec<(f64, f64)>,  // (价格, 数量)
}

// 定义24小时行情数据结构
#[derive(Debug, Clone)]
pub struct MarketTicker {
    pub price_change_percent: f64,  // 24小时价格变动百分比
    pub high_price: f64,            // 24小时最高价
    pub low_price: f64,             // 24小时最低价
    pub last_price: f64,            // 最新价格
    pub volume: f64,                // 24小时成交量
    pub bid_price: f64,             // 买一价
    pub ask_price: f64,             // 卖一价
}

#[allow(dead_code)]
pub trait TradingStrategy {
    fn should_buy(&self, price: f64, depth: Option<&MarketDepth>, ticker: Option<&MarketTicker>) -> bool;
    fn should_sell(&self, price: f64, depth: Option<&MarketDepth>, ticker: Option<&MarketTicker>) -> bool;
}

#[derive(Debug, PartialEq, Clone)]
enum Signal {
    Buy,
    Sell,
    Hold,
}

pub struct MACDStrategy {
    price_history: Vec<f64>,
    fast_period: usize,   // 快线周期 (通常是12)
    slow_period: usize,   // 慢线周期 (通常是26)
    signal_period: usize, // 信号线周期 (通常是9)
    last_signal: Option<Signal>,
    macd_history: Vec<MACD>,
}

impl MACDStrategy {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            price_history: Vec::new(),
            fast_period,
            slow_period,
            signal_period,
            last_signal: None,
            macd_history: Vec::new(),
        }
    }

    pub fn add_price(&mut self, price: f64) {
        self.price_history.push(price);
        if self.price_history.len() > self.slow_period * 2 {
            self.price_history.remove(0);
        }
        self.update_macd();
    }

    fn calculate_ema(&self, period: usize, prices: &[f64]) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for price in prices.iter().skip(1) {
            ema = price * multiplier + ema * (1.0 - multiplier);
        }

        Some(ema)
    }

    fn update_macd(&mut self) {
        if self.price_history.len() < self.slow_period {
            return;
        }

        // 计算MACD线
        let fast_ema = self.calculate_ema(self.fast_period, &self.price_history)
            .unwrap_or_default();
        let slow_ema = self.calculate_ema(self.slow_period, &self.price_history)
            .unwrap_or_default();
        let macd_line = fast_ema - slow_ema;

        // 计算信号线
        let mut macd_values = self.macd_history.iter()
            .map(|m| m.macd)
            .collect::<Vec<f64>>();
        macd_values.push(macd_line);

        let signal_line = self.calculate_ema(self.signal_period, &macd_values)
            .unwrap_or_default();

        // 计算MACD柱状图
        let histogram = macd_line - signal_line;

        self.macd_history.push(MACD {
            macd: macd_line,
            signal: signal_line,
            histogram,
        });

        // 保持历史数据在合理范围内
        if self.macd_history.len() > self.signal_period * 2 {
            self.macd_history.remove(0);
        }
    }

    // 检查动量趋势
    fn check_momentum_trend(&self) -> Option<(bool, f64)> {
        if self.macd_history.len() < 3 {
            return None;
        }

        let current = self.macd_history.last().unwrap();
        let previous = self.macd_history.get(self.macd_history.len() - 2).unwrap();
        let prev_prev = self.macd_history.get(self.macd_history.len() - 3).unwrap();

        // 计算柱状图变化率
        let curr_change = current.histogram - previous.histogram;
        let prev_change = previous.histogram - prev_prev.histogram;
        
        // 计算动量加速度（变化率的变化）
        let momentum_acceleration = curr_change - prev_change;
        
        // 计算当前柱状图相对变化幅度
        let histogram_change_percent = (current.histogram - previous.histogram).abs() 
            / previous.histogram.abs() * 100.0;

        // 判断动量趋势
        let is_increasing = curr_change > 0.0 && prev_change > 0.0 && momentum_acceleration > 0.0;
        let is_decreasing = curr_change < 0.0 && prev_change < 0.0 && momentum_acceleration < 0.0;

        if is_increasing || is_decreasing {
            Some((is_increasing, histogram_change_percent))
        } else {
            None
        }
    }

    fn check_momentum(&self) -> Option<Signal> {
        if self.macd_history.len() < 2 {
            return None;
        }

        let current = self.macd_history.last()?;

        // 检查动量趋势
        if let Some((is_increasing, change_percent)) = self.check_momentum_trend() {
            if is_increasing && change_percent > 3.0 {  // 动量加速上涨且变化超过3%
                if current.macd >= current.signal || 
                   (current.macd - current.signal).abs() / current.signal.abs() < 0.001 {  // 接近金叉
                    return Some(Signal::Buy);
                }
            } else if !is_increasing && change_percent > 3.0 {  // 动量加速下跌且变化超过3%
                if current.macd <= current.signal || 
                   (current.macd - current.signal).abs() / current.signal.abs() < 0.001 {  // 接近死叉
                    return Some(Signal::Sell);
                }
            }
        }

        Some(Signal::Hold)
    }

    // 检查趋势强度
    fn momentum_strength(&self) -> Option<f64> {
        if self.macd_history.len() < 2 {
            return None;
        }

        let current = self.macd_history.last()?;
        let previous = self.macd_history.get(self.macd_history.len() - 2)?;

        Some((current.histogram - previous.histogram).abs())
    }

    // 分析深度数据
    fn analyze_depth(&self, depth: &MarketDepth, current_price: f64) -> (bool, bool) {
        let mut buy_pressure = 0.0;
        let mut sell_pressure = 0.0;
        
        // 计算买卖压力
        let bid_volume: f64 = depth.bids.iter()
            .filter(|(price, _)| *price > current_price * 0.99) // 考虑接近当前价格的订单
            .map(|(_, quantity)| quantity)
            .sum();
            
        let ask_volume: f64 = depth.asks.iter()
            .filter(|(price, _)| *price < current_price * 1.01) // 考虑接近当前价格的订单
            .map(|(_, quantity)| quantity)
            .sum();
            
        // 计算买卖压力比例
        if ask_volume > 0.0 {
            buy_pressure = bid_volume / ask_volume;
        }
        if bid_volume > 0.0 {
            sell_pressure = ask_volume / bid_volume;
        }
        
        // 判断是否有显著的买卖压力
        let strong_buy = buy_pressure > 1.2;  // 买单量超过卖单量20%
        let strong_sell = sell_pressure > 1.2; // 卖单量超过买单量20%
        
        (strong_buy, strong_sell)
    }

    // 分析24小时行情数据
    fn analyze_ticker(&self, ticker: &MarketTicker) -> (bool, bool) {
        // 计算日内波动率
        let volatility = (ticker.high_price - ticker.low_price) / ticker.low_price * 100.0;
        
        // 计算价格在日内区间的位置 (0-100%)
        let price_position = (ticker.last_price - ticker.low_price) / 
            (ticker.high_price - ticker.low_price) * 100.0;
            
        // 计算买卖压力
        let spread = (ticker.ask_price - ticker.bid_price) / ticker.bid_price * 100.0;
        
        // 判断市场状态
        let mut bullish = false;
        let mut bearish = false;
        
        // 价格变动趋势判断
        if ticker.price_change_percent > 0.2 {  // 上涨超过0.2%
            bullish = true;
        } else if ticker.price_change_percent < -0.2 {  // 下跌超过0.2%
            bearish = true;
        }
        
        // 价格位置判断
        if price_position < 20.0 {  // 接近日内低点
            bullish = true;
        } else if price_position > 80.0 {  // 接近日内高点
            bearish = true;
        }
        
        // 波动率判断
        if volatility > 2.0 {  // 日内波动超过2%，增加风险意识
            bullish = bullish && price_position < 40.0;  // 只在较低位置做多
            bearish = bearish && price_position > 60.0;  // 只在较高位置做空
        }
        
        // 买卖压力判断
        if spread > 0.1 {  // 买卖价差过大，表示流动性不足
            bullish = false;
            bearish = false;
        }
        
        (bullish, bearish)
    }
}

impl TradingStrategy for MACDStrategy {
    fn should_buy(&self, price: f64, depth: Option<&MarketDepth>, ticker: Option<&MarketTicker>) -> bool {
        let signal = self.check_momentum();
        let strength = self.momentum_strength();
        
        // 检查深度数据
        let depth_confirms = depth.map(|d| {
            let (strong_buy, _) = self.analyze_depth(d, price);
            strong_buy
        }).unwrap_or(true);
        
        // 检查24小时行情
        let ticker_confirms = ticker.map(|t| {
            let (bullish, _) = self.analyze_ticker(t);
            bullish
        }).unwrap_or(true);

        match (signal, strength) {
            (Some(Signal::Buy), Some(strength)) => {
                // 同时满足MACD信号、深度分析和行情分析
                strength > 0.0001 && 
                self.last_signal != Some(Signal::Buy) &&
                depth_confirms &&
                ticker_confirms
            }
            _ => false
        }
    }

    fn should_sell(&self, price: f64, depth: Option<&MarketDepth>, ticker: Option<&MarketTicker>) -> bool {
        let signal = self.check_momentum();
        let strength = self.momentum_strength();
        
        // 检查深度数据
        let depth_confirms = depth.map(|d| {
            let (_, strong_sell) = self.analyze_depth(d, price);
            strong_sell
        }).unwrap_or(true);
        
        // 检查24小时行情
        let ticker_confirms = ticker.map(|t| {
            let (_, bearish) = self.analyze_ticker(t);
            bearish
        }).unwrap_or(true);

        match (signal, strength) {
            (Some(Signal::Sell), Some(strength)) => {
                // 同时满足MACD信号、深度分析和行情分析
                strength > 0.0001 && 
                self.last_signal != Some(Signal::Sell) &&
                depth_confirms &&
                ticker_confirms
            }
            _ => false
        }
    }
} 