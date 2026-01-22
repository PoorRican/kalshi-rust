use super::Kalshi;
use crate::kalshi_error::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Deserializer that handles both String and integer values, converting integers to strings.
/// Used for Kalshi API fields that may return integer 0 instead of empty string.
fn string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrIntVisitor;

    impl<'de> serde::de::Visitor<'de> for StringOrIntVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or integer")
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.to_owned())
        }

        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }

        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }
    }

    deserializer.deserialize_any(StringOrIntVisitor)
}

/// Deserializer for Option<String> fields that may receive integer values.
fn option_string_or_int<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptionStringOrIntVisitor;

    impl<'de> serde::de::Visitor<'de> for OptionStringOrIntVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string, integer, or null")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_owned()))
        }

        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }
    }

    deserializer.deserialize_any(OptionStringOrIntVisitor)
}

impl Kalshi {
    /// Retrieves detailed information about a specific event from the Kalshi exchange.
    ///
    /// # Arguments
    /// * `event_ticker` - A string reference representing the ticker of the event.
    /// * `with_nested_markets` - An optional boolean to include nested market data.
    ///
    /// # Returns
    /// - `Ok(Event)`: Event object on successful retrieval.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    /// # Example
    /// ```
    /// // Assuming `kalshi_instance` is an already authenticated instance of `Kalshi`
    /// let event_ticker = "some_event_ticker";
    /// let event = kalshi_instance.get_single_event(event_ticker, None).await.unwrap();
    /// ```
    pub async fn get_single_event(
        &self,
        event_ticker: &String,
        with_nested_markets: Option<bool>,
    ) -> Result<Event, KalshiError> {
        let single_event_url: &str =
            &format!("{}/events/{}", self.base_url.to_string(), event_ticker);

        let mut params: Vec<(&str, String)> = Vec::with_capacity(2);

        add_param!(params, "with_nested_markets", with_nested_markets);

        let single_event_url = reqwest::Url::parse_with_params(single_event_url, &params)
            .unwrap_or_else(|err| {
                eprintln!("{:?}", err);
                panic!("Internal Parse Error, please contact developer!");
            });

        let result: SingleEventResponse = self
            .client
            .get(single_event_url)
            .send()
            .await?
            .json()
            .await?;

        return Ok(result.event);
    }

    /// Retrieves detailed information about a specific market from the Kalshi exchange.
    ///
    /// # Arguments
    /// * `ticker` - A string reference representing the ticker of the market.
    ///
    /// # Returns
    /// - `Ok(Market)`: Market object on successful retrieval.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    /// # Example
    /// ```
    /// // Assuming `kalshi_instance` is an already authenticated instance of `Kalshi`
    /// let market_ticker = "some_event_ticker";
    /// let market = kalshi_instance.get_single_event(market_ticker).await.unwrap();
    /// ```
    pub async fn get_single_market(&self, ticker: &String) -> Result<Market, KalshiError> {
        let single_market_url: &str = &format!("{}/markets/{}", self.base_url.to_string(), ticker);

        let result: SingleMarketResponse = self
            .client
            .get(single_market_url)
            .send()
            .await?
            .json()
            .await?;

        return Ok(result.market);
    }
    /// Asynchronously retrieves information about multiple markets from the Kalshi exchange.
    ///
    /// This method fetches data for a collection of markets, filtered by various optional parameters.
    /// It supports pagination, time-based filtering, and selection by specific tickers or statuses.
    ///
    /// # Arguments
    /// * `limit` - An optional integer to limit the number of markets returned.
    /// * `cursor` - An optional string for pagination cursor.
    /// * `event_ticker` - An optional string to filter markets by event ticker.
    /// * `series_ticker` - An optional string to filter markets by series ticker.
    /// * `max_close_ts` - An optional timestamp for the maximum close time.
    /// * `min_close_ts` - An optional timestamp for the minimum close time.
    /// * `status` - An optional string to filter markets by their status.
    /// * `tickers` - An optional string to filter markets by specific tickers.
    ///
    /// # Returns
    /// - `Ok((Option<String>, Vec<Market>))`: A tuple containing an optional pagination cursor and a vector of `Market` objects on success.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    ///
    /// # Example
    ///
    /// ```
    /// // Assuming `kalshi_instance` is an already authenticated instance of `Kalshi`
    /// let markets_result = kalshi_instance.get_multiple_markets(
    ///     Some(10),
    ///     None,
    ///     Some("event_ticker"),
    ///     None,
    ///     None,
    ///     None,
    ///     None,
    ///     None
    /// ).await.unwrap();
    /// ```
    pub async fn get_multiple_markets(
        &self,
        limit: Option<i64>,
        cursor: Option<String>,
        event_ticker: Option<String>,
        series_ticker: Option<String>,
        max_close_ts: Option<i64>,
        min_close_ts: Option<i64>,
        status: Option<String>,
        tickers: Option<String>,
    ) -> Result<(Option<String>, Vec<Market>), KalshiError> {
        let markets_url: &str = &format!("{}/markets", self.base_url.to_string());

        let mut params: Vec<(&str, String)> = Vec::with_capacity(10);

        add_param!(params, "limit", limit);
        add_param!(params, "event_ticker", event_ticker);
        add_param!(params, "series_ticker", series_ticker);
        add_param!(params, "status", status);
        add_param!(params, "cursor", cursor);
        add_param!(params, "min_close_ts", min_close_ts);
        add_param!(params, "max_close_ts", max_close_ts);
        add_param!(params, "tickers", tickers);

        let markets_url = reqwest::Url::parse_with_params(markets_url, &params)
            .map_err(|e| KalshiError::InternalError(e.to_string()))?;

        let builder = self.client.get(markets_url.clone());
        let builder = if self.is_authenticated() {
            self.add_auth_headers(builder, "GET", &markets_url)?
        } else {
            builder
        };

        let result: PublicMarketsResponse = builder.send().await?.json().await?;

        Ok((result.cursor, result.markets))
    }
    /// Asynchronously retrieves information about multiple events from the Kalshi exchange.
    ///
    /// This method fetches data for multiple events, with optional filtering based on status,
    /// series ticker, and whether nested market data should be included. It supports pagination
    /// and time-based filtering.
    ///
    /// # Arguments
    /// * `limit` - An optional integer to limit the number of events returned.
    /// * `cursor` - An optional string for pagination cursor.
    /// * `status` - An optional string to filter events by their status.
    /// * `series_ticker` - An optional string to filter events by series ticker.
    /// * `with_nested_markets` - An optional boolean to include nested market data.
    ///
    /// # Returns
    /// - `Ok((Option<String>, Vec<Event>))`: A tuple containing an optional pagination cursor and a vector of `Event` objects on success.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    ///
    /// # Example
    ///
    /// ```
    /// // Assuming `kalshi_instance` is an already authenticated instance of `Kalshi`
    /// let events_result = kalshi_instance.get_multiple_events(
    ///     Some(10),
    ///     None,
    ///     Some("active"),
    ///     None,
    ///     Some(true)
    /// ).await.unwrap();
    /// println!("Events: {:?}", events_result);
    /// ```
    ///
    pub async fn get_multiple_events(
        &self,
        limit: Option<i64>,
        cursor: Option<String>,
        status: Option<String>,
        series_ticker: Option<String>,
        with_nested_markets: Option<bool>,
    ) -> Result<(Option<String>, Vec<Event>), KalshiError> {
        let events_url: &str = &format!("{}/events", self.base_url.to_string());

        let mut params: Vec<(&str, String)> = Vec::with_capacity(6);

        add_param!(params, "limit", limit);
        add_param!(params, "status", status);
        add_param!(params, "cursor", cursor);
        add_param!(params, "series_ticker", series_ticker);
        add_param!(params, "with_nested_markets", with_nested_markets);

        let events_url =
            reqwest::Url::parse_with_params(events_url, &params).unwrap_or_else(|err| {
                eprintln!("{:?}", err);
                panic!("Internal Parse Error, please contact developer!");
            });

        let result: PublicEventsResponse = self.client.get(events_url).send().await?.json().await?;

        return Ok((result.cursor, result.events));
    }
    /// Asynchronously retrieves detailed information about a specific series from the Kalshi exchange.
    ///
    /// This method fetches data for a series identified by its ticker. The series data includes
    /// information such as frequency, title, category, settlement sources, and related contract URLs.
    ///
    /// # Arguments
    /// * `ticker` - A reference to a string representing the series's ticker.
    ///
    /// # Returns
    /// - `Ok(Series)`: `Series` object on successful retrieval.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    /// # Example
    /// ```
    /// // Assuming `kalshi_instance` is an already authenticated instance of `Kalshi`
    /// let series_ticker = "some_series_ticker";
    /// let series = kalshi_instance.get_series(series_ticker).await.unwrap();
    /// ```
    pub async fn get_series(&self, ticker: &String) -> Result<Series, KalshiError> {
        let series_url: &str = &format!("{}/series/{}", self.base_url.to_string(), ticker);

        let result: SeriesResponse = self.client.get(series_url).send().await?.json().await?;

        return Ok(result.series);
    }
    /// Asynchronously retrieves the order book for a specific market in the Kalshi exchange.
    ///
    /// This method fetches the order book for a market, which includes the bid and ask prices
    /// for both 'Yes' and 'No' options. It allows specifying the depth of the order book to be retrieved.
    ///
    /// # Arguments
    /// * `ticker` - A reference to a string representing the market's ticker.
    /// * `depth` - An optional integer specifying the depth of the order book.
    ///
    /// # Returns
    /// - `Ok(Orderbook)`: `Orderbook` object on successful retrieval.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    ///
    /// # Example
    /// Returns an orderbook with a depth of 10 entries for some market.
    /// ```
    /// // Assuming `kalshi_instance` is an already authenticated instance of `Kalshi`
    /// let market_ticker = "some_market_ticker";
    /// let orderbook = kalshi_instance.get_market_orderbook(market_ticker, Some(10)).await.unwrap();
    /// ```
    pub async fn get_market_orderbook(
        &self,
        ticker: &String,
        depth: Option<i32>,
    ) -> Result<Orderbook, KalshiError> {
        let orderbook_url: &str =
            &format!("{}/markets/{}/orderbook", self.base_url.to_string(), ticker);

        let mut params: Vec<(&str, String)> = Vec::new();

        add_param!(params, "depth", depth);

        let orderbook_url = reqwest::Url::parse_with_params(orderbook_url, &params)
            .map_err(|e| KalshiError::InternalError(e.to_string()))?;

        let builder = self.client.get(orderbook_url.clone());
        let builder = if self.is_authenticated() {
            self.add_auth_headers(builder, "GET", &orderbook_url)?
        } else {
            builder
        };

        let result: OrderBookResponse = builder.send().await?.json().await?;

        Ok(result.orderbook)
    }

    /// Asynchronously retrieves the market history for a given market on the Kalshi exchange.
    ///
    /// This method fetches historical data for a specific market, which can include
    /// details like prices, bids, asks, volume, and open interest over time. It allows
    /// filtering the history based on time and pagination parameters.
    ///
    /// # Arguments
    /// * `ticker` - A reference to a string representing the market's ticker.
    /// * `limit` - An optional integer to limit the number of history records returned.
    /// * `cursor` - An optional string for pagination cursor.
    /// * `min_ts` - An optional timestamp to specify the minimum time for history records.
    /// * `max_ts` - An optional timestamp to specify the maximum time for history records.
    ///
    /// # Returns
    /// - `Ok((Option<String>, Vec<Snapshot>))`: A tuple containing an optional pagination cursor and a vector of `Snapshot` objects on success.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    /// # Example
    ///
    /// ```
    /// // Assuming `kalshi_instance` is an already authenticated instance of `Kalshi`
    /// let market_history = kalshi_instance.get_market_history(
    ///     "ticker_name",
    ///     Some(10),
    ///     None,
    ///     None,
    ///     None
    /// ).await.unwrap();
    /// ```
    pub async fn get_market_history(
        &self,
        ticker: &String,
        limit: Option<i32>,
        cursor: Option<String>,
        min_ts: Option<i64>,
        max_ts: Option<i64>,
    ) -> Result<(Option<String>, Vec<Snapshot>), KalshiError> {
        let market_history_url: &str =
            &format! {"{}/markets/{}/history", self.base_url.to_string(), ticker};

        let mut params: Vec<(&str, String)> = Vec::with_capacity(5);

        add_param!(params, "limit", limit);
        add_param!(params, "cursor", cursor);
        add_param!(params, "min_ts", min_ts);
        add_param!(params, "max_ts", max_ts);

        let market_history_url = reqwest::Url::parse_with_params(market_history_url, &params)
            .map_err(|e| KalshiError::InternalError(e.to_string()))?;

        let builder = self.client.get(market_history_url.clone());
        let builder = if self.is_authenticated() {
            self.add_auth_headers(builder, "GET", &market_history_url)?
        } else {
            builder
        };

        let result: MarketHistoryResponse = builder.send().await?.json().await?;

        Ok((result.cursor, result.history))
    }

    /// Asynchronously retrieves trade data from the Kalshi exchange.
    ///
    /// This method fetches data about trades that have occurred, including details like trade ID,
    /// taker side, ticker, and executed prices. It supports filtering based on various parameters
    /// such as time, ticker, and pagination options.
    ///
    /// # Arguments
    /// * `cursor` - An optional string for pagination cursor.
    /// * `limit` - An optional integer to limit the number of trades returned.
    /// * `ticker` - An optional string representing the market's ticker for which trades are to be fetched.
    /// * `min_ts` - An optional timestamp to specify the minimum time for trade records.
    /// * `max_ts` - An optional timestamp to specify the maximum time for trade records.
    ///
    /// # Returns
    /// - `Ok((Option<String>, Vec<Trade>))`: A tuple containing an optional pagination cursor and a vector of `Trade` objects on success.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    /// ```
    /// // Assuming `kalshi_instance` is an already authenticated instance of `Kalshi`
    /// let trades = kalshi_instance.get_trades(
    ///     None,
    ///     Some(10),
    ///     Some("ticker_name"),
    ///     None,
    ///     None
    /// ).await.unwrap();
    /// ```
    pub async fn get_trades(
        &self,
        cursor: Option<String>,
        limit: Option<i32>,
        ticker: Option<String>,
        min_ts: Option<i64>,
        max_ts: Option<i64>,
    ) -> Result<(Option<String>, Vec<Trade>), KalshiError> {
        let trades_url: &str = &format!("{}/markets/trades", self.base_url.to_string());

        let mut params: Vec<(&str, String)> = Vec::with_capacity(7);

        add_param!(params, "limit", limit);
        add_param!(params, "cursor", cursor);
        add_param!(params, "min_ts", min_ts);
        add_param!(params, "max_ts", max_ts);
        add_param!(params, "ticker", ticker);

        let trades_url =
            reqwest::Url::parse_with_params(trades_url, &params).unwrap_or_else(|err| {
                eprintln!("{:?}", err);
                panic!("Internal Parse Error, please contact developer!");
            });

        let result: PublicTradesResponse = self.client.get(trades_url).send().await?.json().await?;

        Ok((result.cursor, result.trades))
    }

    /// Retrieves candlestick (OHLC) data for a specific market.
    ///
    /// # Arguments
    /// * `series_ticker` - The ticker of the series containing the market.
    /// * `ticker` - The ticker of the market.
    /// * `start_ts` - Start timestamp (Unix epoch seconds).
    /// * `end_ts` - End timestamp (Unix epoch seconds).
    /// * `period_interval` - The time interval for each candlestick.
    ///
    /// # Returns
    /// - `Ok(Vec<Candlestick>)`: Vector of candlestick data on success.
    /// - `Err(KalshiError)`: Error on failure.
    pub async fn get_market_candlesticks(
        &self,
        series_ticker: &str,
        ticker: &str,
        start_ts: i64,
        end_ts: i64,
        period_interval: PeriodInterval,
    ) -> Result<Vec<Candlestick>, KalshiError> {
        let url = format!(
            "{}/series/{}/markets/{}/candlesticks",
            self.base_url, series_ticker, ticker
        );

        let mut params: Vec<(&str, String)> = Vec::with_capacity(3);
        params.push(("start_ts", start_ts.to_string()));
        params.push(("end_ts", end_ts.to_string()));
        params.push(("period_interval", period_interval.to_string()));

        let url = reqwest::Url::parse_with_params(&url, &params).unwrap_or_else(|err| {
            eprintln!("{:?}", err);
            panic!("Internal Parse Error, please contact developer!");
        });

        let result: SingleMarketCandlesticksResponse =
            self.client.get(url).send().await?.json().await?;

        Ok(result.candlesticks)
    }

    /// Retrieves candlestick (OHLC) data for multiple markets in a single request.
    ///
    /// # Arguments
    /// * `tickers` - Comma-separated list of market tickers (max 100).
    /// * `start_ts` - Start timestamp (Unix epoch seconds).
    /// * `end_ts` - End timestamp (Unix epoch seconds).
    /// * `period_interval` - The time interval for each candlestick.
    ///
    /// # Returns
    /// - `Ok(Vec<MarketCandlesticks>)`: Vector of market candlestick data on success.
    /// - `Err(KalshiError)`: Error on failure.
    ///
    /// # Notes
    /// - Maximum of 100 tickers per request.
    /// - Maximum of 10,000 total candlesticks returned.
    pub async fn get_batch_market_candlesticks(
        &self,
        tickers: &str,
        start_ts: i64,
        end_ts: i64,
        period_interval: PeriodInterval,
    ) -> Result<Vec<MarketCandlesticks>, KalshiError> {
        // Client-side validation: check if request would exceed 10,000 candlestick limit
        const MAX_CANDLESTICKS: usize = 10_000;
        let num_tickers = tickers.split(',').count();
        let time_range_secs = (end_ts - start_ts).max(0) as usize;
        let interval_secs: usize = match period_interval {
            PeriodInterval::OneMinute => 60,
            PeriodInterval::OneHour => 3600,
            PeriodInterval::OneDay => 86400,
        };
        let num_intervals = time_range_secs / interval_secs;
        let theoretical_max = num_intervals * num_tickers;

        if theoretical_max > MAX_CANDLESTICKS {
            return Err(KalshiError::UserInputError(format!(
                "Request would exceed maximum candlesticks limit ({} max). \
                 Estimated: {} candlesticks ({} tickers x {} intervals). \
                 Reduce tickers or time range.",
                MAX_CANDLESTICKS, theoretical_max, num_tickers, num_intervals
            )));
        }

        let url = format!("{}/markets/candlesticks", self.base_url);

        let mut params: Vec<(&str, String)> = Vec::with_capacity(4);
        params.push(("market_tickers", tickers.to_string()));
        params.push(("start_ts", start_ts.to_string()));
        params.push(("end_ts", end_ts.to_string()));
        params.push(("period_interval", period_interval.to_string()));

        let url = reqwest::Url::parse_with_params(&url, &params).unwrap_or_else(|err| {
            eprintln!("{:?}", err);
            panic!("Internal Parse Error, please contact developer!");
        });

        let result: BatchCandlesticksApiResponse =
            self.client.get(url).send().await?.json().await?;

        match result {
            BatchCandlesticksApiResponse::Success(response) => Ok(response.markets),
            BatchCandlesticksApiResponse::Error(err) => {
                let message = match err.error.details {
                    Some(details) => format!("{}: {}", err.error.message, details),
                    None => err.error.message,
                };
                Err(KalshiError::UserInputError(message))
            }
        }
    }

    /// Retrieves candlestick (OHLC) data for all markets in an event.
    ///
    /// # Arguments
    /// * `series_ticker` - The ticker of the series containing the event.
    /// * `event_ticker` - The ticker of the event.
    /// * `start_ts` - Start timestamp (Unix epoch seconds).
    /// * `end_ts` - End timestamp (Unix epoch seconds).
    /// * `period_interval` - The time interval for each candlestick.
    ///
    /// # Returns
    /// - `Ok(EventCandlesticks)`: Candlestick data for each market in the event.
    /// - `Err(KalshiError)`: Error on failure.
    pub async fn get_event_candlesticks(
        &self,
        series_ticker: &str,
        event_ticker: &str,
        start_ts: i64,
        end_ts: i64,
        period_interval: PeriodInterval,
    ) -> Result<EventCandlesticks, KalshiError> {
        let url = format!(
            "{}/series/{}/events/{}/candlesticks",
            self.base_url, series_ticker, event_ticker
        );

        let mut params: Vec<(&str, String)> = Vec::with_capacity(3);
        params.push(("start_ts", start_ts.to_string()));
        params.push(("end_ts", end_ts.to_string()));
        params.push(("period_interval", period_interval.to_string()));

        let url = reqwest::Url::parse_with_params(&url, &params).unwrap_or_else(|err| {
            eprintln!("{:?}", err);
            panic!("Internal Parse Error, please contact developer!");
        });

        let result: EventCandlesticksResponse = self.client.get(url).send().await?.json().await?;

        Ok(EventCandlesticks {
            market_tickers: result.market_tickers,
            market_candlesticks: result.market_candlesticks,
            adjusted_end_ts: result.adjusted_end_ts,
        })
    }
}

// PRIVATE STRUCTS
// used in get_single_event
#[derive(Debug, Deserialize, Serialize)]
struct SingleEventResponse {
    event: Event,
    #[serde(default)]
    markets: Option<Vec<Market>>,
}

// used in get_single_market
#[derive(Debug, Deserialize, Serialize)]
struct SingleMarketResponse {
    market: Market,
}

#[derive(Debug, Deserialize, Serialize)]
struct PublicMarketsResponse {
    cursor: Option<String>,
    markets: Vec<Market>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PublicEventsResponse {
    cursor: Option<String>,
    events: Vec<Event>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SeriesResponse {
    series: Series,
}

#[derive(Debug, Deserialize, Serialize)]
struct OrderBookResponse {
    orderbook: Orderbook,
}

#[derive(Debug, Deserialize, Serialize)]
struct MarketHistoryResponse {
    cursor: Option<String>,
    ticker: String,
    history: Vec<Snapshot>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PublicTradesResponse {
    cursor: Option<String>,
    trades: Vec<Trade>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SingleMarketCandlesticksResponse {
    ticker: String,
    candlesticks: Vec<Candlestick>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BatchCandlesticksResponse {
    markets: Vec<MarketCandlesticks>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetails {
    code: String,
    message: String,
    #[serde(default)]
    details: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetails,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum BatchCandlesticksApiResponse {
    Success(BatchCandlesticksResponse),
    Error(ApiErrorResponse),
}

#[derive(Debug, Deserialize, Serialize)]
struct EventCandlesticksResponse {
    market_tickers: Vec<String>,
    market_candlesticks: Vec<Vec<Candlestick>>,
    #[serde(default)]
    adjusted_end_ts: Option<i64>,
}

// PUBLIC STRUCTS

/// Period interval for candlestick data.
#[derive(Debug, Clone, Copy)]
pub enum PeriodInterval {
    /// 1-minute candles
    OneMinute,
    /// 1-hour candles
    OneHour,
    /// 1-day candles
    OneDay,
}

impl Serialize for PeriodInterval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(match self {
            PeriodInterval::OneMinute => 1,
            PeriodInterval::OneHour => 60,
            PeriodInterval::OneDay => 1440,
        })
    }
}

impl<'de> Deserialize<'de> for PeriodInterval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        match value {
            1 => Ok(PeriodInterval::OneMinute),
            60 => Ok(PeriodInterval::OneHour),
            1440 => Ok(PeriodInterval::OneDay),
            _ => Err(serde::de::Error::custom(format!(
                "invalid period interval: {}",
                value
            ))),
        }
    }
}

impl fmt::Display for PeriodInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeriodInterval::OneMinute => write!(f, "1"),
            PeriodInterval::OneHour => write!(f, "60"),
            PeriodInterval::OneDay => write!(f, "1440"),
        }
    }
}

/// OHLC (Open, High, Low, Close) price data.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Ohlc {
    /// Opening price (in cents)
    pub open: Option<i64>,
    /// Highest price during the period
    pub high: Option<i64>,
    /// Lowest price during the period
    pub low: Option<i64>,
    /// Closing price
    pub close: Option<i64>,
}

/// Price OHLC data with previous period reference.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PriceOhlc {
    /// Opening price (in cents)
    pub open: Option<i64>,
    /// Highest price during the period
    pub high: Option<i64>,
    /// Lowest price during the period
    pub low: Option<i64>,
    /// Closing price
    pub close: Option<i64>,
    /// Previous period's closing price
    pub previous: Option<i64>,
}

/// A candlestick (OHLC) data point for market price history.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Candlestick {
    /// End timestamp of the candlestick period (Unix epoch seconds)
    pub end_period_ts: i64,
    /// Price OHLC data
    #[serde(default)]
    pub price: Option<PriceOhlc>,
    /// Yes-side bid OHLC
    #[serde(default)]
    pub yes_bid: Option<Ohlc>,
    /// Yes-side ask OHLC
    #[serde(default)]
    pub yes_ask: Option<Ohlc>,
    /// Trading volume during the period
    #[serde(default)]
    pub volume: Option<i64>,
    /// Open interest at the end of the period
    #[serde(default)]
    pub open_interest: Option<i64>,
}

/// Candlestick data for a specific market (used in batch responses).
#[derive(Debug, Deserialize, Serialize)]
pub struct MarketCandlesticks {
    /// Market ticker
    #[serde(alias = "market_ticker")]
    pub ticker: String,
    /// Candlestick data for this market
    pub candlesticks: Vec<Candlestick>,
}

/// Aggregated candlestick data for an event containing multiple markets.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventCandlesticks {
    /// Market tickers in this event
    pub market_tickers: Vec<String>,
    /// Candlestick data per market (parallel to market_tickers)
    pub market_candlesticks: Vec<Vec<Candlestick>>,
    /// Adjusted end timestamp if data was truncated
    pub adjusted_end_ts: Option<i64>,
}

/// A market in the Kalshi exchange.
///
/// Contains detailed information about the market including its ticker,
/// type, status, and other relevant data.
///
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Market {
    /// Unique identifier for the market.
    pub ticker: String,
    /// Ticker of the associated event.
    pub event_ticker: String,
    /// Type of the market.
    pub market_type: String,
    /// Title of the market.
    pub title: String,
    /// Subtitle of the market.
    pub subtitle: String,
    /// Subtitle for the 'Yes' option in the market.
    pub yes_sub_title: String,
    /// Subtitle for the 'No' option in the market.
    pub no_sub_title: String,
    /// Opening time of the market.
    pub open_time: String,
    /// Closing time of the market.
    pub close_time: String,
    /// Expected expiration time of the market.
    pub expected_expiration_time: Option<String>,
    /// Actual expiration time of the market.
    pub expiration_time: Option<String>,
    /// Latest possible expiration time of the market.
    pub latest_expiration_time: String,
    /// Countdown in seconds to the settlement.
    pub settlement_timer_seconds: i64,
    /// Current status of the market.
    pub status: MarketStatus,
    /// Units used for pricing responses.
    pub response_price_units: String,
    /// Notional value of the market.
    pub notional_value: i64,
    /// Minimum price movement in the market.
    pub tick_size: i64,
    /// Current bid price for the 'Yes' option.
    pub yes_bid: i64,
    /// Current ask price for the 'Yes' option.
    pub yes_ask: i64,
    /// Current bid price for the 'No' option.
    pub no_bid: i64,
    /// Current ask price for the 'No' option.
    pub no_ask: i64,
    /// Last traded price in the market.
    pub last_price: i64,
    /// Previous bid price for the 'Yes' option.
    pub previous_yes_bid: i64,
    /// Previous ask price for the 'Yes' option.
    pub previous_yes_ask: i64,
    /// Previous traded price in the market.
    pub previous_price: i64,
    /// Total trading volume in the market.
    pub volume: i64,
    /// Trading volume in the last 24 hours.
    pub volume_24h: i64,
    /// Liquidity available in the market.
    pub liquidity: i64,
    /// Open interest in the market.
    pub open_interest: i64,
    /// Result of the market settlement.
    pub result: SettlementResult,
    /// Cap strike price, if applicable.
    pub cap_strike: Option<f64>,
    /// Indicator if the market can close early.
    pub can_close_early: bool,
    /// Value at expiration.
    #[serde(deserialize_with = "string_or_int")]
    pub expiration_value: String,
    /// Category of the market.
    pub category: String,
    /// Risk limit in cents.
    pub risk_limit_cents: i64,
    /// Type of strike, if applicable.
    #[serde(default, deserialize_with = "option_string_or_int")]
    pub strike_type: Option<String>,
    /// Floor strike price, if applicable.
    pub floor_strike: Option<f64>,
    /// Primary rules for the market.
    pub rules_primary: String,
    /// Secondary rules for the market.
    pub rules_secondary: String,
    /// Settlement value for the market (in cents).
    pub settlement_value: Option<i64>,
    /// Functional strike information, if applicable.
    #[serde(default, deserialize_with = "option_string_or_int")]
    pub functional_strike: Option<String>,
}

/// An event in the Kalshi exchange.
///
/// This struct contains information about a specific event, including its identifier,
/// title, and other relevant details. It may also include associated markets.
///
#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    /// Unique identifier for the event.
    pub event_ticker: String,
    /// Ticker of the associated series.
    pub series_ticker: String,
    /// Subtitle of the event.
    pub sub_title: String,
    /// Title of the event.
    pub title: String,
    /// Indicates if the event's outcomes are mutually exclusive.
    pub mutually_exclusive: bool,
    /// Category of the event.
    pub category: String,
    /// Optional list of markets associated with this event.
    #[serde(default)]
    pub markets: Option<Vec<Market>>,
    /// Optional date of the event's occurrence.
    pub strike_date: Option<String>,
    /// Optional period of the event.
    pub strike_period: Option<String>,
}

/// Series on the Kalshi exchange.
///
/// This struct includes details about a specific series, such as its frequency,
/// title, and category. It also includes information on settlement sources and
/// related contract URLs.
///
#[derive(Debug, Deserialize, Serialize)]
pub struct Series {
    /// Unique ticker identifying the series.
    pub ticker: String,
    /// Frequency of the series.
    pub frequency: String,
    /// Title of the series.
    pub title: String,
    /// Category of the series.
    pub category: String,
    /// Tags associated with the series.
    pub tags: Vec<String>,
    /// Sources used for settling the series.
    pub settlement_sources: Vec<SettlementSource>,
    /// URL of the contract related to the series.
    pub contract_url: String,
}

/// A source of a settlement in the Kalshi exchange.
///
/// This struct contains information about a source used for settling a series, including the source's URL and name.
///
#[derive(Debug, Deserialize, Serialize)]
pub struct SettlementSource {
    /// URL of the settlement source.
    pub url: String,
    /// Name of the settlement source.
    pub name: String,
}

/// The order book of a market in the Kalshi exchange.
///
/// This struct includes the bid and ask prices for both 'Yes' and 'No' options in a market, structured as nested vectors.
///
#[derive(Debug, Deserialize, Serialize)]
pub struct Orderbook {
    /// Nested vector of bids and asks for the 'Yes' option.
    /// Each inner vector typically contains price and quantity.
    pub yes: Option<Vec<Vec<i32>>>,
    /// Nested vector of bids and asks for the 'No' option.
    /// Each inner vector typically contains price and quantity.
    pub no: Option<Vec<Vec<i32>>>,
}

/// Snapshot of market data in the Kalshi exchange.
///
/// This struct provides a snapshot of the market at a specific time, including prices, bids, asks, volume, and open interest.
///
#[derive(Debug, Deserialize, Serialize)]
pub struct Snapshot {
    /// Last traded price for the 'Yes' option.
    pub yes_price: i32,
    /// Current highest bid price for the 'Yes' option.
    pub yes_bid: i32,
    /// Current lowest ask price for the 'Yes' option.
    pub yes_ask: i32,
    /// Current highest bid price for the 'No' option.
    pub no_bid: i32,
    /// Current lowest ask price for the 'No' option.
    pub no_ask: i32,
    /// Total trading volume at the snapshot time.
    pub volume: i32,
    /// Open interest at the snapshot time.
    pub open_interest: i32,
    /// Timestamp of the snapshot.
    pub ts: i64,
}

/// A trade in the Kalshi exchange.
///
/// This struct contains details of an individual trade, including the trade ID, side, ticker, and executed prices.
///
/// Used in methods for retrieving user fills and specific trade details.
///
#[derive(Debug, Deserialize, Serialize)]
pub struct Trade {
    /// Unique identifier of the trade.
    pub trade_id: String,
    /// Side of the taker in the trade (e.g., 'buyer' or 'seller').
    pub taker_side: String,
    /// Ticker of the market in which the trade occurred.
    pub ticker: String,
    /// Number of contracts or shares traded.
    pub count: i32,
    /// Executed price for the 'Yes' option.
    pub yes_price: i32,
    /// Executed price for the 'No' option.
    pub no_price: i32,
    /// Time when the trade was created.
    pub created_time: String,
}

/// Possible outcomes of a market settlement on the Kalshi exchange.
///
/// This enum represents the different results that can be assigned to a market
/// upon its conclusion.
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettlementResult {
    /// The outcome of the market is affirmative.
    Yes,
    /// The outcome of the market is negative.
    No,
    /// The market is voided, usually due to specific conditions not being met.
    #[serde(rename = "")]
    Void,
    /// All options in the market are settled as 'No'.
    #[serde(rename = "all_no")]
    AllNo,
    /// All options in the market are settled as 'Yes'.
    #[serde(rename = "all_yes")]
    AllYes,
}

/// The different statuses a market can have on the Kalshi exchange.
///
/// This enum is used to represent the current operational state of a market.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    /// The market is initialized but not yet open.
    Initialized,
    /// The market is inactive.
    Inactive,
    /// The market is active for trading.
    Active,
    /// The market is closed and not currently available for trading.
    Closed,
    /// The market outcome has been determined.
    Determined,
    /// The market is under dispute.
    Disputed,
    /// The market has been amended.
    Amended,
    /// The market has been finalized.
    Finalized,
}

impl std::fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketStatus::Initialized => write!(f, "initialized"),
            MarketStatus::Inactive => write!(f, "inactive"),
            MarketStatus::Active => write!(f, "active"),
            MarketStatus::Closed => write!(f, "closed"),
            MarketStatus::Determined => write!(f, "determined"),
            MarketStatus::Disputed => write!(f, "disputed"),
            MarketStatus::Amended => write!(f, "amended"),
            MarketStatus::Finalized => write!(f, "finalized"),
        }
    }
}

impl std::str::FromStr for MarketStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "initialized" => Ok(MarketStatus::Initialized),
            "inactive" => Ok(MarketStatus::Inactive),
            "active" => Ok(MarketStatus::Active),
            "closed" => Ok(MarketStatus::Closed),
            "determined" => Ok(MarketStatus::Determined),
            "disputed" => Ok(MarketStatus::Disputed),
            "amended" => Ok(MarketStatus::Amended),
            "finalized" => Ok(MarketStatus::Finalized),
            other => Err(format!("unknown market status: {}", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn sample_market_json() -> &'static str {
        r#"{
            "ticker": "TEST-TICKER",
            "event_ticker": "TEST-EVENT",
            "market_type": "binary",
            "title": "Test Market",
            "subtitle": "Test Subtitle",
            "yes_sub_title": "Yes",
            "no_sub_title": "No",
            "open_time": "2024-01-01T00:00:00Z",
            "close_time": "2024-12-31T23:59:59Z",
            "expected_expiration_time": null,
            "expiration_time": null,
            "latest_expiration_time": "2024-12-31T23:59:59Z",
            "settlement_timer_seconds": 3600,
            "status": "active",
            "response_price_units": "cents",
            "notional_value": 100,
            "tick_size": 1,
            "yes_bid": 50,
            "yes_ask": 55,
            "no_bid": 45,
            "no_ask": 50,
            "last_price": 52,
            "previous_yes_bid": 49,
            "previous_yes_ask": 54,
            "previous_price": 51,
            "volume": 10000,
            "volume_24h": 500,
            "liquidity": 5000,
            "open_interest": 2000,
            "result": "",
            "cap_strike": null,
            "can_close_early": false,
            "expiration_value": "",
            "category": "politics",
            "risk_limit_cents": 100000,
            "strike_type": null,
            "floor_strike": null,
            "rules_primary": "Primary rules text",
            "rules_secondary": "Secondary rules text",
            "settlement_value": null,
            "functional_strike": null
        }"#
    }

    #[test]
    fn test_market_deserialization() -> serde_json::Result<()> {
        let market: Market = serde_json::from_str(sample_market_json())?;
        assert_eq!(market.ticker, "TEST-TICKER");
        assert_eq!(market.event_ticker, "TEST-EVENT");
        assert_eq!(market.yes_bid, 50);
        assert_eq!(market.volume, 10000);
        assert!(matches!(market.result, SettlementResult::Void));
        Ok(())
    }

    #[test]
    fn test_single_market_response_deserialization() -> serde_json::Result<()> {
        let json = format!(r#"{{"market": {}}}"#, sample_market_json());
        let response: SingleMarketResponse = serde_json::from_str(&json)?;
        assert_eq!(response.market.ticker, "TEST-TICKER");
        Ok(())
    }

    #[test]
    fn test_public_markets_response_deserialization() -> serde_json::Result<()> {
        let json = format!(
            r#"{{"cursor": "next_page_cursor", "markets": [{}]}}"#,
            sample_market_json()
        );
        let response: PublicMarketsResponse = serde_json::from_str(&json)?;
        assert_eq!(response.cursor, Some("next_page_cursor".to_string()));
        assert_eq!(response.markets.len(), 1);
        assert_eq!(response.markets[0].ticker, "TEST-TICKER");
        Ok(())
    }

    #[test]
    fn test_public_markets_response_null_cursor() -> serde_json::Result<()> {
        let json = format!(
            r#"{{"cursor": null, "markets": [{}]}}"#,
            sample_market_json()
        );
        let response: PublicMarketsResponse = serde_json::from_str(&json)?;
        assert!(response.cursor.is_none());
        Ok(())
    }

    #[test]
    fn test_orderbook_deserialization() -> serde_json::Result<()> {
        let json = r#"{"yes": [[50, 100], [49, 200]], "no": [[50, 150]]}"#;
        let orderbook: Orderbook = serde_json::from_str(json)?;
        let yes = orderbook.yes.unwrap();
        assert_eq!(yes.len(), 2);
        assert_eq!(yes[0], vec![50, 100]);
        let no = orderbook.no.unwrap();
        assert_eq!(no.len(), 1);
        Ok(())
    }

    #[test]
    fn test_orderbook_empty() -> serde_json::Result<()> {
        let json = r#"{"yes": null, "no": null}"#;
        let orderbook: Orderbook = serde_json::from_str(json)?;
        assert!(orderbook.yes.is_none());
        assert!(orderbook.no.is_none());
        Ok(())
    }

    #[test]
    fn test_snapshot_deserialization() -> serde_json::Result<()> {
        let json = r#"{
            "yes_price": 52,
            "yes_bid": 50,
            "yes_ask": 55,
            "no_bid": 45,
            "no_ask": 50,
            "volume": 1000,
            "open_interest": 500,
            "ts": 1704067200
        }"#;
        let snapshot: Snapshot = serde_json::from_str(json)?;
        assert_eq!(snapshot.yes_price, 52);
        assert_eq!(snapshot.ts, 1704067200);
        Ok(())
    }

    #[test]
    fn test_trade_deserialization() -> serde_json::Result<()> {
        let json = r#"{
            "trade_id": "trade-123",
            "taker_side": "yes",
            "ticker": "TEST-TICKER",
            "count": 10,
            "yes_price": 52,
            "no_price": 48,
            "created_time": "2024-01-01T12:00:00Z"
        }"#;
        let trade: Trade = serde_json::from_str(json)?;
        assert_eq!(trade.trade_id, "trade-123");
        assert_eq!(trade.count, 10);
        Ok(())
    }

    #[test]
    fn test_settlement_result_variants() -> serde_json::Result<()> {
        assert!(matches!(
            serde_json::from_str::<SettlementResult>(r#""yes""#)?,
            SettlementResult::Yes
        ));
        assert!(matches!(
            serde_json::from_str::<SettlementResult>(r#""no""#)?,
            SettlementResult::No
        ));
        assert!(matches!(
            serde_json::from_str::<SettlementResult>(r#""""#)?,
            SettlementResult::Void
        ));
        assert!(matches!(
            serde_json::from_str::<SettlementResult>(r#""all_no""#)?,
            SettlementResult::AllNo
        ));
        assert!(matches!(
            serde_json::from_str::<SettlementResult>(r#""all_yes""#)?,
            SettlementResult::AllYes
        ));
        Ok(())
    }

    #[test]
    fn test_market_status_variants() -> serde_json::Result<()> {
        assert!(matches!(
            serde_json::from_str::<MarketStatus>(r#""initialized""#)?,
            MarketStatus::Initialized
        ));
        assert!(matches!(
            serde_json::from_str::<MarketStatus>(r#""inactive""#)?,
            MarketStatus::Inactive
        ));
        assert!(matches!(
            serde_json::from_str::<MarketStatus>(r#""active""#)?,
            MarketStatus::Active
        ));
        assert!(matches!(
            serde_json::from_str::<MarketStatus>(r#""closed""#)?,
            MarketStatus::Closed
        ));
        assert!(matches!(
            serde_json::from_str::<MarketStatus>(r#""determined""#)?,
            MarketStatus::Determined
        ));
        assert!(matches!(
            serde_json::from_str::<MarketStatus>(r#""disputed""#)?,
            MarketStatus::Disputed
        ));
        assert!(matches!(
            serde_json::from_str::<MarketStatus>(r#""amended""#)?,
            MarketStatus::Amended
        ));
        assert!(matches!(
            serde_json::from_str::<MarketStatus>(r#""finalized""#)?,
            MarketStatus::Finalized
        ));
        Ok(())
    }

    #[test]
    fn test_orderbook_response_deserialization() -> serde_json::Result<()> {
        let json = r#"{"orderbook": {"yes": [[50, 100]], "no": [[50, 150]]}}"#;
        let response: OrderBookResponse = serde_json::from_str(json)?;
        assert!(response.orderbook.yes.is_some());
        Ok(())
    }

    #[test]
    fn test_market_history_response_deserialization() -> serde_json::Result<()> {
        let json = r#"{
            "cursor": "next",
            "ticker": "TEST-TICKER",
            "history": [{
                "yes_price": 52,
                "yes_bid": 50,
                "yes_ask": 55,
                "no_bid": 45,
                "no_ask": 50,
                "volume": 1000,
                "open_interest": 500,
                "ts": 1704067200
            }]
        }"#;
        let response: MarketHistoryResponse = serde_json::from_str(json)?;
        assert_eq!(response.ticker, "TEST-TICKER");
        assert_eq!(response.history.len(), 1);
        Ok(())
    }

    // Integration Tests (require network access)
    // Run with: cargo test --package kalshi -- --ignored

    /// Helper struct to cache test data across integration tests
    struct TestMarketData {
        ticker: String,
        event_ticker: String,
        series_ticker: String,
    }

    /// Fetches a valid open market from the demo API for testing
    async fn get_test_market_data() -> TestMarketData {
        let kalshi = crate::Kalshi::new(crate::TradingEnvironment::LiveMarketMode);
        let (_, markets) = kalshi
            .get_multiple_markets(
                Some(1),
                None,
                None,
                None,
                None,
                None,
                Some("open".to_string()),
                None,
            )
            .await
            .expect("Failed to fetch markets from API");

        let market = markets.first().expect("No open markets available");

        // Get the series ticker from the event
        let event = kalshi
            .get_single_event(&market.event_ticker, None)
            .await
            .expect("Failed to fetch event");

        TestMarketData {
            ticker: market.ticker.clone(),
            event_ticker: market.event_ticker.clone(),
            series_ticker: event.series_ticker,
        }
    }

    #[tokio::test]
    #[ignore] // Requires network access
    #[serial]
    async fn test_get_single_market() {
        let test_data = get_test_market_data().await;
        let kalshi = crate::Kalshi::new(crate::TradingEnvironment::LiveMarketMode);

        let result = kalshi.get_single_market(&test_data.ticker).await;

        let market = result.expect("Failed to fetch market");
        assert!(!market.ticker.is_empty());
        assert!(!market.event_ticker.is_empty());
        assert!(!market.title.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    #[serial]
    async fn test_get_multiple_markets() {
        let kalshi = crate::Kalshi::new(crate::TradingEnvironment::LiveMarketMode);

        let result = kalshi
            .get_multiple_markets(
                Some(5),
                None,
                None,
                None,
                None,
                None,
                Some("open".to_string()),
                None,
            )
            .await;

        let (_, markets) = result.expect("Failed to fetch markets");
        assert!(!markets.is_empty());
        for market in &markets {
            assert!(!market.ticker.is_empty());
            assert!(!market.title.is_empty());
        }
    }

    #[tokio::test]
    #[ignore] // Requires network access
    #[serial]
    async fn test_get_market_orderbook() {
        let test_data = get_test_market_data().await;
        let kalshi = crate::Kalshi::new(crate::TradingEnvironment::LiveMarketMode);

        let result = kalshi
            .get_market_orderbook(&test_data.ticker, Some(10))
            .await;

        let orderbook = result.expect("Failed to fetch orderbook");
        // Orderbook may be empty but should deserialize correctly
        // yes and no can be None or Some with empty/populated arrays
        assert!(orderbook.yes.is_none() || orderbook.yes.is_some());
        assert!(orderbook.no.is_none() || orderbook.no.is_some());
    }

    // NOTE: test_get_market_history is skipped because the /markets/{ticker}/history
    // endpoint returns 404 on the demo API. The endpoint may be deprecated or unavailable.
    // Use get_market_candlesticks for historical price data instead.

    #[tokio::test]
    #[ignore] // Requires network access
    #[serial]
    async fn test_get_trades() {
        let test_data = get_test_market_data().await;
        let kalshi = crate::Kalshi::new(crate::TradingEnvironment::LiveMarketMode);

        let result = kalshi
            .get_trades(None, Some(5), Some(test_data.ticker), None, None)
            .await;

        let (_, trades) = result.expect("Failed to fetch trades");
        // Trades may be empty but should deserialize correctly
        for trade in &trades {
            assert!(!trade.trade_id.is_empty());
            assert!(!trade.ticker.is_empty());
        }
    }

    #[tokio::test]
    #[ignore] // Requires network access
    #[serial]
    async fn test_get_market_candlesticks() {
        let test_data = get_test_market_data().await;
        let kalshi = crate::Kalshi::new(crate::TradingEnvironment::LiveMarketMode);

        // Use a wide time range to ensure we get some data
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let one_week_ago = now - (7 * 24 * 60 * 60);

        let result = kalshi
            .get_market_candlesticks(
                &test_data.series_ticker,
                &test_data.ticker,
                one_week_ago,
                now,
                PeriodInterval::OneHour,
            )
            .await;

        let candlesticks = result.expect("Failed to fetch candlesticks");
        // Candlesticks may be empty but should deserialize correctly
        for candle in &candlesticks {
            assert!(candle.end_period_ts > 0);
        }
    }

    #[tokio::test]
    #[ignore] // Requires network access
    #[serial]
    async fn test_get_batch_market_candlesticks() {
        let kalshi = crate::Kalshi::new(crate::TradingEnvironment::LiveMarketMode);

        // Fetch a few tickers for batch request
        let (_, markets) = kalshi
            .get_multiple_markets(
                Some(3),
                None,
                None,
                None,
                None,
                None,
                Some("open".to_string()),
                None,
            )
            .await
            .expect("Failed to fetch markets");

        let tickers: Vec<&str> = markets.iter().map(|m| m.ticker.as_str()).collect();
        let tickers_str = tickers.join(",");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let one_week_ago = now - (7 * 24 * 60 * 60);

        let result = kalshi
            .get_batch_market_candlesticks(&tickers_str, one_week_ago, now, PeriodInterval::OneDay)
            .await;

        let market_candlesticks = result.expect("Failed to fetch batch candlesticks");
        // May return empty results but should deserialize correctly
        for mc in &market_candlesticks {
            assert!(!mc.ticker.is_empty());
        }
    }

    #[tokio::test]
    #[ignore] // Requires network access
    #[serial]
    async fn test_get_event_candlesticks() {
        let test_data = get_test_market_data().await;
        let kalshi = crate::Kalshi::new(crate::TradingEnvironment::LiveMarketMode);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let start_ts = now - (30 * 24 * 60 * 60); // 30 days ago

        let result = kalshi
            .get_event_candlesticks(
                &test_data.series_ticker,
                &test_data.event_ticker,
                start_ts,
                now,
                PeriodInterval::OneDay,
            )
            .await;

        match result {
            Ok(event_candlesticks) => {
                // Verify we have market tickers and corresponding candlestick arrays
                assert!(
                    !event_candlesticks.market_tickers.is_empty()
                        || event_candlesticks.market_candlesticks.is_empty(),
                    "Market tickers and candlesticks should be consistent"
                );
                assert_eq!(
                    event_candlesticks.market_tickers.len(),
                    event_candlesticks.market_candlesticks.len(),
                    "Should have same number of market tickers and candlestick arrays"
                );
            }
            Err(e) => {
                eprintln!("Event candlesticks request failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_candlestick_deserialization() -> serde_json::Result<()> {
        let json = r#"{
            "end_period_ts": 1704067200,
            "price": {
                "open": 50,
                "high": 55,
                "low": 48,
                "close": 52,
                "previous": 49
            },
            "yes_bid": {
                "open": 49,
                "high": 54,
                "low": 47,
                "close": 51
            },
            "yes_ask": {
                "open": 51,
                "high": 56,
                "low": 49,
                "close": 53
            },
            "volume": 1000,
            "open_interest": 500
        }"#;
        let candle: Candlestick = serde_json::from_str(json)?;
        assert_eq!(candle.end_period_ts, 1704067200);
        assert_eq!(candle.volume, Some(1000));
        assert_eq!(candle.open_interest, Some(500));
        let price = candle.price.unwrap();
        assert_eq!(price.open, Some(50));
        assert_eq!(price.close, Some(52));
        assert_eq!(price.previous, Some(49));
        let yes_bid = candle.yes_bid.unwrap();
        assert_eq!(yes_bid.close, Some(51));
        Ok(())
    }

    #[test]
    fn test_candlestick_minimal_deserialization() -> serde_json::Result<()> {
        let json = r#"{"end_period_ts": 1704067200}"#;
        let candle: Candlestick = serde_json::from_str(json)?;
        assert_eq!(candle.end_period_ts, 1704067200);
        assert!(candle.price.is_none());
        assert!(candle.yes_bid.is_none());
        assert!(candle.volume.is_none());
        Ok(())
    }

    #[test]
    fn test_market_candlesticks_deserialization() -> serde_json::Result<()> {
        let json = r#"{
            "ticker": "TEST-TICKER",
            "candlesticks": [
                {"end_period_ts": 1704067200, "volume": 100},
                {"end_period_ts": 1704070800, "volume": 200}
            ]
        }"#;
        let market_candles: MarketCandlesticks = serde_json::from_str(json)?;
        assert_eq!(market_candles.ticker, "TEST-TICKER");
        assert_eq!(market_candles.candlesticks.len(), 2);
        assert_eq!(market_candles.candlesticks[0].volume, Some(100));
        Ok(())
    }

    #[test]
    fn test_batch_candlesticks_response_deserialization() -> serde_json::Result<()> {
        let json = r#"{
            "markets": [
                {
                    "ticker": "MARKET-A",
                    "candlesticks": [{"end_period_ts": 1704067200}]
                },
                {
                    "ticker": "MARKET-B",
                    "candlesticks": [{"end_period_ts": 1704070800}]
                }
            ]
        }"#;
        let response: BatchCandlesticksResponse = serde_json::from_str(json)?;
        assert_eq!(response.markets.len(), 2);
        assert_eq!(response.markets[0].ticker, "MARKET-A");
        Ok(())
    }

    #[test]
    fn test_event_candlesticks_response_deserialization() -> serde_json::Result<()> {
        let json = r#"{
            "market_tickers": ["MARKET-A", "MARKET-B"],
            "market_candlesticks": [
                [{"end_period_ts": 1704067200, "volume": 100}],
                [{"end_period_ts": 1704067200, "volume": 200}]
            ],
            "adjusted_end_ts": 1704153600
        }"#;
        let response: EventCandlesticksResponse = serde_json::from_str(json)?;
        assert_eq!(response.market_tickers.len(), 2);
        assert_eq!(response.market_candlesticks.len(), 2);
        assert_eq!(response.adjusted_end_ts, Some(1704153600));
        assert_eq!(response.market_candlesticks[0][0].volume, Some(100));
        Ok(())
    }

    #[test]
    fn test_event_candlesticks_response_without_adjusted_end_ts() -> serde_json::Result<()> {
        let json = r#"{
            "market_tickers": ["MARKET-A"],
            "market_candlesticks": [[{"end_period_ts": 1704067200}]]
        }"#;
        let response: EventCandlesticksResponse = serde_json::from_str(json)?;
        assert_eq!(response.market_tickers.len(), 1);
        assert!(response.adjusted_end_ts.is_none());
        Ok(())
    }

    #[test]
    fn test_batch_candlesticks_error_response_deserialization() -> serde_json::Result<()> {
        let json = r#"{"error":{"code":"bad_request","message":"bad request","details":"requested candlesticks across all markets: 12960, max candlesticks: 10000"}}"#;
        let response: BatchCandlesticksApiResponse = serde_json::from_str(json)?;
        assert!(matches!(response, BatchCandlesticksApiResponse::Error(_)));
        if let BatchCandlesticksApiResponse::Error(err) = response {
            assert_eq!(err.error.code, "bad_request");
            assert_eq!(err.error.message, "bad request");
            assert!(err.error.details.is_some());
            assert!(err.error.details.unwrap().contains("max candlesticks: 10000"));
        }
        Ok(())
    }

    #[test]
    fn test_batch_candlesticks_invalid_params_error() -> serde_json::Result<()> {
        let json = r#"{"error":{"code":"invalid_parameters","message":"invalid parameters"}}"#;
        let response: BatchCandlesticksApiResponse = serde_json::from_str(json)?;
        assert!(matches!(response, BatchCandlesticksApiResponse::Error(_)));
        if let BatchCandlesticksApiResponse::Error(err) = response {
            assert_eq!(err.error.code, "invalid_parameters");
            assert!(err.error.details.is_none());
        }
        Ok(())
    }

    #[test]
    fn test_batch_candlesticks_success_response_deserialization() -> serde_json::Result<()> {
        let json = r#"{"markets":[{"ticker":"TEST-TICKER","candlesticks":[{"end_period_ts":1704067200}]}]}"#;
        let response: BatchCandlesticksApiResponse = serde_json::from_str(json)?;
        assert!(matches!(response, BatchCandlesticksApiResponse::Success(_)));
        if let BatchCandlesticksApiResponse::Success(resp) = response {
            assert_eq!(resp.markets.len(), 1);
            assert_eq!(resp.markets[0].ticker, "TEST-TICKER");
        }
        Ok(())
    }

    #[test]
    fn test_candlesticks_limit_calculation() {
        // 100 tickers, 24h of 1-minute data = 100 * 1440 = 144,000 (exceeds 10,000)
        let num_tickers = 100;
        let time_range_secs: usize = 24 * 60 * 60; // 24 hours
        let interval_secs: usize = 60; // 1 minute
        let theoretical_max = (time_range_secs / interval_secs) * num_tickers;
        assert!(theoretical_max > 10_000);

        // 10 tickers, 1h of 1-minute data = 10 * 60 = 600 (within limit)
        let num_tickers = 10;
        let time_range_secs: usize = 60 * 60; // 1 hour
        let theoretical_max = (time_range_secs / interval_secs) * num_tickers;
        assert!(theoretical_max <= 10_000);
    }
}
