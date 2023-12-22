use reqwest::Client;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::error::Error;
use std::fmt;
use tokio::time::{self, Duration};

fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringToF64;

    impl<'de> Visitor<'de> for StringToF64 {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string that can be parsed into a f64")
        }

        fn visit_str<E>(self, value: &str) -> Result<f64, E>
        where
            E: de::Error,
        {
            value.parse::<f64>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_string(StringToF64)
}

fn deserialize_string_to_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringTof32;

    impl<'de> Visitor<'de> for StringTof32 {
        type Value = f32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string that can be parsed into a f32")
        }

        fn visit_str<E>(self, value: &str) -> Result<f32, E>
        where
            E: de::Error,
        {
            value.parse::<f32>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_string(StringTof32)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Orderbook {
    pair: String,
    base: String,
    quote: String,
    #[serde(deserialize_with = "deserialize_string_to_f32")]
    min_amount: f32,
    #[serde(deserialize_with = "deserialize_string_to_f32")]
    tick_size: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Quote {
    pub timestamp: String,

    #[serde(deserialize_with = "deserialize_string_to_f32")]
    pub bid_price: f32,

    #[serde(deserialize_with = "deserialize_string_to_f64")]
    bid_amount: f64,

    #[serde(deserialize_with = "deserialize_string_to_f32")]
    pub ask_price: f32,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    ask_amount: f64,
}

pub async fn get_order_books(url: &str) -> Result<Vec<Orderbook>, Box<dyn Error>> {
    // setup a request client
    let client: Client = Client::new();

    // make fetch request
    let response = client.get(url).send().await?;

    // check if response is success and return response
    if response.status().is_success() {
        let body = response.text().await?;
        let order_books: Vec<Orderbook> = serde_json::from_str(&body)?;
        Ok(order_books)
    } else {
        Err(Box::new(response.error_for_status().unwrap_err()))
    }
}

pub async fn get_quote_price(url: &str) -> Result<Quote, Box<dyn Error>> {
    let client: Client = Client::new();

    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let quote: Quote = serde_json::from_str(&body)?;
        Ok(quote)
    } else {
        Err(Box::new(response.error_for_status().unwrap_err()))
    }
}

pub async fn get_quote_price_interval(time: u64, pair: &str, quotes: &mut Vec<Quote>) {
    let interval = Duration::from_secs(time);
    let mut interval_timer = time::interval(interval);
    let url = format!(
        "https://cax.piccadilly.autonity.org/api/orderbooks/{}/quote",
        pair
    );
    loop {
        interval_timer.tick().await;

        match get_quote_price(&url).await {
            Ok(quote) => {
                println!("Received quote: {:?}", &quote);
                quotes.push(quote);
            }
            Err(e) => {
                eprintln!("Error fetching quote: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{get_order_books, get_quote_price};
    #[tokio::test]
    async fn test() {
        let orderbooks =
            get_order_books("https://cax.piccadilly.autonity.org/api/orderbooks").await;

        println!("orderbooks: {:?}", orderbooks);

        let url = format!(
            "https://cax.piccadilly.autonity.org/api/orderbooks/{}/quote",
            "ATN-USD"
        );

        let quote = get_quote_price(url).await;
        println!("quote: {:?}", quote)
    }
}
