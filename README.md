# [UNOFFICIAL] Criptomarket API

This is a fork of [gallegogt/cryptomkt-rs](https://github.com/gallegogt/cryptomkt-rs). It allows do make non blocking calls. 


# Examples
```rust 
use cryptomkt::{Client, OrderType};
const API_KEY: &'static str = "<API_KEY>";
const API_SECRET: &'static str = "<API SECRET>";

#[tokio::main]
async fn main() {
    let client = Client::new(API_KEY, API_SECRET);

    // Get all markets available
    let markets = client.get_markets().await;
    for m in markets.iter() {
        println!("{}", m.get_name());

        // Get the current ticker for the market
        match m.get_current_ticker().await {
            Ok(ticker) => {
                println!("{:?}", ticker);
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }

        println!("------- Orders ------");
        match m.get_orders_book(OrderType::Buy, 0, 20).await {
            Ok(orders) => {
                println!("{:?}", orders);
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }

        println!("------- Trades ------");
        match m.get_trades("2018-05-15", "2018-05-16", 0, 20).await {
            Ok(trades) => {
                println!("{:?}", trades);
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}

```

# Support My Efforts

[Gallegogt](https://github.com/gallegogt) programmed this lib for fun and he does his best effort to support those that have issues with it, please return the favor and support him.

[![paypal](https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif)](https://www.paypal.me/reiloygt)
