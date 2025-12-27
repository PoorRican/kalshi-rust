use kalshi::{Action, Kalshi, OrderType, Side, TradingEnvironment};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Initialize with API key authentication
    // Set env vars: KALSHI_DEMO_API_KEY_ID and KALSHI_DEMO_PRIVATE_KEY (path or PEM content)
    let kalshi = Kalshi::new_with_api_key_from_env(
        TradingEnvironment::DemoMode,
        "KALSHI_DEMO_API_KEY_ID",
        "KALSHI_DEMO_PRIVATE_KEY",
    )
        .expect("Failed to initialize Kalshi client");

    let ticker = "HIGHNY-23NOV13-T51".to_string();

    // Fetch market data
    let market = kalshi.get_single_market(&ticker).await.unwrap();
    println!("Market: {:?}", market.title);

    // Fetch orderbook
    let orderbook = kalshi.get_market_orderbook(&ticker, Some(1)).await.unwrap();
    println!("Orderbook: {:?}", orderbook);

    // Place and cancel an order
    let order = kalshi
        .create_order(Action::Buy, None, 1, Side::Yes, ticker, OrderType::Limit, None, None, None, None, Some(5))
        .await
        .unwrap();

    let cancelled = kalshi.cancel_order(&order.order_id).await.unwrap();
    println!("Cancelled: {:?}", cancelled);
}
