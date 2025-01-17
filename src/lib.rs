//!
//! cryptomkt-rs provides a simple and powerful implementation for CryptoMarket API.
//!
//! Example where the last ticker is shown for each market available in the exchange market Criptomarket
//!
//! ```
//! use cryptomkt::{Client, OrderType};
//!
//! const API_KEY: &'static str = "<API_KEY>";
//! const API_SECRET: &'static str = "<API SECRET>";
//! 
//! #[tokio::main]
//! async fn main() {
//!
//!     let client = Client::new(API_KEY, API_SECRET);
//!
//!     // Get the markets available in the exchange
//!     let markets = client.get_markets().await;
//!     for m in markets.iter() {
//!         println!("{}", m.get_name());
//!
//!         // GET current Ticker
//!         match m.get_current_ticker().await {
//!             Ok(ticker) => {
//!                 println!("{:?}", ticker);
//!             }
//!             Err(e) => {
//!                 println!("{:?}", e);
//!             }
//!         }
//!     }
//! }
//! ```


mod api;
mod client;
mod internal;
mod market;

pub use crate::api::{CryptoMktApi, RequestMethod};
pub use crate::client::Client;
pub use crate::internal::models;
pub use crate::internal::response;
pub use crate::market::{Market, OrderType};
