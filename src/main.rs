use ethereum_types::{U256};
use serde::Deserialize;
use serde_json::{Value};
use jsonpath_rust::{JsonPathQuery};

trait OracleSource {
    fn fetch(&self, params: Vec<u8>) -> Result<U256, reqwest::Error>;
}

#[derive(Debug)]
pub struct TimeSourceAdapter {
    pub name: String,
}

impl TimeSourceAdapter {
    pub fn new(name: String) -> Self {
        TimeSourceAdapter {
            name,
        }
    }
}

impl OracleSource for TimeSourceAdapter {
    fn fetch(&self, _params: Vec<u8>) -> Result<U256, reqwest::Error> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(U256::from(time))
    }
}

#[derive(Debug)]
pub struct RngSourceAdapter {
    pub name: String,
}

impl RngSourceAdapter {
    pub fn new(name: String) -> Self {
        RngSourceAdapter {
            name,
        }
    }
}

impl OracleSource for RngSourceAdapter {
    fn fetch(&self, _params: Vec<u8>) -> Result<U256, reqwest::Error> {
        //Ok(U256::from(new_stdrng().unwrap().gen::<u64>()))
        Ok(U256::from(1))
    }
}

#[derive(Debug)]
pub struct ExchangeSourceAdapter {
    pub name: String,
    pub url: String,
    pub params: Vec<String>,
    pub jsonpath: String,
    pub decimal: u32,
    pub bases: Vec<String>,
    pub quotes: Vec<String>,
}

impl ExchangeSourceAdapter {
    pub fn new(
        name: String,
        url: String,
        params: Vec<String>,
        jsonpath: String,
        decimal: u32,
        bases: Vec<String>,
        quotes: Vec<String>
    ) -> Self {
        ExchangeSourceAdapter {
            name,
            url,
            params,
            jsonpath,
            decimal,
            bases,
            quotes,
        }
    }
}

impl OracleSource for ExchangeSourceAdapter {
    fn fetch(&self, params: Vec<u8>) -> Result<U256, reqwest::Error> {
        let mut url = self.url.clone();
        for param in &self.params {
            url = url.replacen("{}", &param, 1);
        }
        url = url.replacen("{}", &self.quotes[params[0] as usize], 1);
        url = url.replacen("{}", &self.bases[params[1] as usize], 1);

        let resp = reqwest::blocking::get(&url)?.text();
        if resp.is_err() {
            return Err(resp.err().unwrap());
        }

        let rpc_result: Value = serde_json::from_str(&resp.unwrap()).unwrap();
        let data = &rpc_result.path(&self.jsonpath).unwrap()[0];
        if data.is_string() {
            let price = data.as_str().unwrap().parse::<f64>().unwrap() * 10_f64.powf(self.decimal.into());
            return Ok(ethereum_types::U256::from(price as u64));
        }
        else {
            let price = data.as_f64().unwrap() * 10_f64.powf(self.decimal.into());
            return Ok(ethereum_types::U256::from(price as u64));
        }
    }
}

#[derive(Debug)]
pub struct CustomSourceAdapter {
    pub url: String,
    pub jsonpath: String,
    pub decimal: u32,
}

impl CustomSourceAdapter {
    pub fn new(url: String, jsonpath: String, decimal: u32) -> Self {
        CustomSourceAdapter {
            url,
            jsonpath,
            decimal,
        }
    }
}

impl OracleSource for CustomSourceAdapter {
    fn fetch(&self, _params: Vec<u8>) -> Result<U256, reqwest::Error> {
        let resp = reqwest::blocking::get(&self.url)?.text();
        if resp.is_err() {
            return Err(resp.err().unwrap());
        }

        let rpc_result: Value = serde_json::from_str(&resp.unwrap()).unwrap();
        let data = &rpc_result.path(&self.jsonpath).unwrap()[0];
        if data.is_string() {
            let price = data.as_str().unwrap().parse::<f64>().unwrap() * 10_f64.powf(self.decimal.into());
            return Ok(ethereum_types::U256::from(price as u64));
        }
        else {
            let price = data.as_f64().unwrap() * 10_f64.powf(self.decimal.into());
            return Ok(ethereum_types::U256::from(price as u64));
        }
    }
}


// TOML loader
#[derive(Debug, Deserialize)]
struct SourceList {
    sources: Vec<SourceConfig>,
}

#[derive(Debug, Deserialize)]
struct SourceConfig {
    name: String,
    url: String,
    params: Vec<String>,
    jsonpath: String,
    decimal: u32,
    bases: Vec<String>,
    quotes: Vec<String>,
}

impl SourceList {
    pub fn new(sources: String) -> Self {
        let list: SourceList = toml::from_str(&sources).unwrap();

        for x in &list.sources {
            println!("{:?}", x.url);
        }

        list
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_adapter_new() {
        let adapter = TimeSourceAdapter::new("time".to_string());
        assert_eq!(adapter.name, "time");
    }

    #[test]
    fn test_rng_adapter_new() {
        let adapter = RngSourceAdapter::new("rng".to_string());
        assert_eq!(adapter.name, "rng");
    }

    #[test]
    fn test_exchange_adapter_new() {
        let adapter = ExchangeSourceAdapter::new(
            "cryptocompare".to_string(),
            "https://min-api.cryptocompare.com/data/price?api_key={}&fsym={}&tsyms={}".to_string(),
            ["d4cf504725efe27b71ec7d213f5db583ef56e88cfbf437a3483d6bb43e9839ab".to_string()].to_vec(),
            "$..*".to_string(),
            12,
            ["BTC".to_string(), "ETH".to_string(), "USDT".to_string(), "USDC".to_string(), "BNB".to_string(), "XRP".to_string(), "BUSD".to_string()].to_vec(),
            [
                "BTC".to_string(),
                "ETH".to_string(),
                "USDT".to_string(),
                "USDC".to_string(),
                "BNB".to_string(),
                "XRP".to_string(),
                "BUSD".to_string(),
                "DOGE".to_string(),
                "ADA".to_string(),
                "MATIC".to_string(),
            ].to_vec()
        );

        assert_eq!(adapter.name, "cryptocompare");
    }

    #[test]
    fn test_custom_adapter_new() {
        let adapter = CustomSourceAdapter::new(
            "https://min-api.cryptocompare.com/data/price?api_key=d4cf504725efe27b71ec7d213f5db583ef56e88cfbf437a3483d6bb43e9839ab&fsym=BTC&tsyms=ETH".to_string(),
            "$..*".to_string(),
            12,
        );

        assert_eq!(adapter.decimal, 12);
    }

    #[test]
    fn test_time_adapter_fetch() {
        let adapter = TimeSourceAdapter::new("time".to_string());
        let result = adapter.fetch(vec![]);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_rng_adapter_fetch() {
        let adapter = RngSourceAdapter::new("rng".to_string());
        let result = adapter.fetch(vec![]);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_exchange_adapter_fetch() {
        let adapter = ExchangeSourceAdapter::new(
            "cryptocompare".to_string(),
            "https://min-api.cryptocompare.com/data/price?api_key={}&fsym={}&tsyms={}".to_string(),
            ["d4cf504725efe27b71ec7d213f5db583ef56e88cfbf437a3483d6bb43e9839ab".to_string()].to_vec(),
            "$..*".to_string(),
            12,
            ["BTC".to_string(), "ETH".to_string(), "USDT".to_string(), "USDC".to_string(), "BNB".to_string(), "XRP".to_string(), "BUSD".to_string()].to_vec(),
            [
                "BTC".to_string(),
                "ETH".to_string(),
                "USDT".to_string(),
                "USDC".to_string(),
                "BNB".to_string(),
                "XRP".to_string(),
                "BUSD".to_string(),
                "DOGE".to_string(),
                "ADA".to_string(),
                "MATIC".to_string(),
            ].to_vec()
        );
        let result = adapter.fetch([1, 2].to_vec());
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_custom_adapter_fetch() {
        let adapter = CustomSourceAdapter::new(
            "https://min-api.cryptocompare.com/data/price?api_key=d4cf504725efe27b71ec7d213f5db583ef56e88cfbf437a3483d6bb43e9839ab&fsym=BTC&tsyms=ETH".to_string(),
            "$..*".to_string(),
            12,
        );
        let result = adapter.fetch(vec![]);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_load_list() {
        let list = SourceList::new(
            r#"
            [[sources]]
            name = "cryptocompare"
            url= "https://min-api.cryptocompare.com/data/price?api_key={}&fsym={}&tsyms={}"
            params = ["d4cf504725efe27b71ec7d213f5db583ef56e88cfbf437a3483d6bb43e9839ab"]
            jsonpath = "$..*"
            decimal = 12
            bases = ["BTC", "ETH", "USDT", "USDC", "BNB", "XRP", "BUSD"]
            quotes = ["BTC", "ETH", "USDT", "USDC", "BNB", "XRP", "BUSD", "DOGE", "ADA", "MATIC"]

            [[sources]]
            name = "binance"
            url= "https://fapi.binance.com/fapi/v1/ticker/price?symbol={}{}"
            params = []
            jsonpath = "$.price"
            decimal = 12
            bases = ["USDT"]
            quotes = ["BTC"]

            [[sources]]
            name = "kucoin"
            url= "https://openapi-sandbox.kucoin.com/api/v1/mark-price/{}-{}/current"
            params = []
            jsonpath = "$.data.value"
            decimal = 12
            bases = ["BTC"]
            quotes = ["USDT"]
            "#.to_string(),
        );

        for source in &list.sources {
            let adapter = ExchangeSourceAdapter::new(
                source.name.clone(),
                source.url.clone(),
                source.params.clone(),
                source.jsonpath.clone(),
                source.decimal,
                source.bases.clone(),
                source.quotes.clone(),
            );
            let result = adapter.fetch([0, 0].to_vec());
            assert_eq!(result.is_ok(), true);
        }
        assert_eq!(list.sources.len(), 3);
    }
}