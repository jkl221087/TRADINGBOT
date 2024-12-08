#[allow(dead_code)]
pub struct Config {
    pub api_key: String,
    pub api_secret: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            api_key: std::env::var("BINGX_API_KEY").unwrap_or_default(),
            api_secret: std::env::var("BINGX_API_SECRET").unwrap_or_default(),
        }
    }
} 