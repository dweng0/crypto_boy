mod api;
mod cli;
use api::{get_quote_price, Quote};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Debug)]
struct PricePoint {
    x: String,
    y: f32,
}

impl PricePoint {
    fn new(quote: &Quote) -> PricePoint {
        let x = quote.timestamp.clone();
        let y = (quote.bid_price + quote.ask_price) / 2.0;
        PricePoint { x, y }
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut price_points: Vec<PricePoint> = Vec::new();
    // Spawn a new thread for fetching quotes
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(get_quote_price_interval(5, "ATN-USD", tx));
    });

    // Main thread: Process the quotes received from the channel
    for quote in rx {
        println!("Received quote in main: {:?}", quote);
        let price_point = PricePoint::new(&quote);
        price_points.push(price_point);

        //todo, should get the mid price from the quotes and add that to an array instead
        println!("quotes: {:?}", price_points);
    }
}

async fn get_quote_price_interval(time: u64, pair: &str, tx: mpsc::Sender<Quote>) {
    let interval = Duration::from_secs(time);
    let url = format!(
        "https://cax.piccadilly.autonity.org/api/orderbooks/{}/quote",
        pair
    );
    loop {
        // Call the async function get_quote_price
        match get_quote_price(&url).await {
            Ok(quote) => {
                println!("Received quote: {:?}", &quote);
                if tx.send(quote).is_err() {
                    eprintln!("Error sending quote");
                    break; // Exit loop if receiver is dropped
                }
            }
            Err(e) => {
                eprintln!("Error fetching quote: {}", e);
            }
        }

        // Sleep for the interval duration
        tokio::time::sleep(interval).await;
    }
}

// Adjust the signature of `get_quote_price_interval` to use the sender
