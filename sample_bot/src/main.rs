use dotenv::dotenv;
use kalshi::Kalshi;
use std::env;

extern crate kalshi;

enum APIType {
    Live,
    Demo,
}

/// Retrieves API key credentials from environment variables.
///
/// Environment variables:
/// - KALSHI_API_KEY_ID: Your Kalshi API key ID
/// - KALSHI_PRIVATE_KEY: Path to PEM file or raw PEM content
fn retrieve_api_key_credentials(setting: APIType) -> Result<(String, String), std::io::Error> {
    let (key_id_var, pem_var) = match setting {
        APIType::Live => ("KALSHI_LIVE_API_KEY_ID", "KALSHI_LIVE_PRIVATE_KEY"),
        APIType::Demo => ("KALSHI_DEMO_API_KEY_ID", "KALSHI_DEMO_PRIVATE_KEY"),
    };

    let key_id = env::var(key_id_var).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Environment variable {} not set", key_id_var),
        )
    })?;

    let pem = env::var(pem_var).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Environment variable {} not set", pem_var),
        )
    })?;

    Ok((key_id, pem))
}

#[allow(deprecated)]
#[tokio::main]
async fn main() {
    dotenv().ok();

    // ============================================
    // RECOMMENDED: API Key Authentication
    // ============================================
    //
    // Set the following environment variables:
    // - KALSHI_DEMO_API_KEY_ID: Your API key ID
    // - KALSHI_DEMO_PRIVATE_KEY: Path to your .pem file or raw PEM content
    //
    // Example .env file:
    //   KALSHI_DEMO_API_KEY_ID=your-api-key-id
    //   KALSHI_DEMO_PRIVATE_KEY=/path/to/your/private_key.pem

    let kalshi_instance = match retrieve_api_key_credentials(APIType::Demo) {
        Ok((key_id, pem)) => {
            println!("Using API key authentication");
            match Kalshi::new_with_api_key(
                kalshi::TradingEnvironment::DemoMode,
                &key_id,
                &pem,
            ) {
                Ok(instance) => instance,
                Err(e) => {
                    eprintln!("Failed to initialize with API key: {:?}", e);
                    eprintln!("Falling back to legacy login authentication...");
                    create_legacy_instance().await
                }
            }
        }
        Err(_) => {
            println!("API key credentials not found, using legacy login authentication");
            create_legacy_instance().await
        }
    };

    // Example usage - works with both authentication methods
    let new_york_ticker = "HIGHNY-23NOV13-T51".to_string();

    let nytemp_market_data = kalshi_instance
        .get_single_market(&new_york_ticker)
        .await
        .unwrap();
    println!("Market data: {:?}", nytemp_market_data.title);

    let nytemp_market_orderbook = kalshi_instance
        .get_market_orderbook(&new_york_ticker, Some(1))
        .await
        .unwrap();
    println!("Orderbook: {:?}", nytemp_market_orderbook);

    let bought_order = kalshi_instance
        .create_order(
            kalshi::Action::Buy,
            None,
            1,
            kalshi::Side::Yes,
            new_york_ticker,
            kalshi::OrderType::Limit,
            None,
            None,
            None,
            None,
            Some(5),
        )
        .await
        .unwrap();

    let ny_order_id = bought_order.order_id.clone();

    let cancelled_order = kalshi_instance.cancel_order(&ny_order_id).await.unwrap();
    println!("{:?}", cancelled_order);
}

/// Creates a Kalshi instance using the legacy email/password login.
/// This method is deprecated - use API key authentication instead.
#[allow(deprecated)]
async fn create_legacy_instance() -> Kalshi {
    let password = env::var("DEMO_PASSWORD").unwrap_or_else(|_| "dummy".to_string());
    let username = env::var("DEMO_USER_NAME").unwrap_or_else(|_| "dummy".to_string());

    let mut kalshi_instance = Kalshi::new(kalshi::TradingEnvironment::DemoMode);

    if let Err(e) = kalshi_instance.login(&username, &password).await {
        eprintln!("Login failed: {:?}", e);
    }

    kalshi_instance
}
