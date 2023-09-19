use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::{MexcApiClient, MexcApiClientWithAuthentication, MexcApiEndpoint};
use crate::v3::{ApiV3Result, QueryWithSignature};
use crate::v3::enums::{OrderSide, OrderType, TradeType};

#[derive(Debug)]
pub struct OrderParams<'a> {
    pub symbol: &'a str,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Option<BigDecimal>,
    pub quote_order_quantity: Option<BigDecimal>,
    pub price: Option<BigDecimal>,
    pub new_client_order_id: Option<&'a str>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuery<'a> {
    pub symbol: &'a str,
    pub side: OrderSide,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<BigDecimal>,
    #[serde(rename = "quoteOrderQty", skip_serializing_if = "Option::is_none")]
    pub quote_order_quantity: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<&'a str>,
    /// Max 60000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

impl<'a> From<OrderParams<'a>> for OrderQuery<'a> {
    fn from(params: OrderParams<'a>) -> Self {
        Self {
            symbol: params.symbol,
            side: params.side,
            order_type: params.order_type,
            quantity: params.quantity,
            quote_order_quantity: params.quote_order_quantity,
            price: params.price,
            new_client_order_id: params.new_client_order_id,
            recv_window: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderOutput {
    pub symbol: String,
    pub order_id: String,
    pub order_list_id: Option<i32>,
    pub price: BigDecimal,
    pub orig_qty: BigDecimal,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub transact_time: DateTime<Utc>,
}


#[async_trait]
pub trait OrderEndpoint {
    async fn order(&self, params: OrderParams<'_>) -> ApiV3Result<OrderOutput>;
}

#[async_trait]
impl OrderEndpoint for MexcApiClientWithAuthentication {
    async fn order(&self, params: OrderParams<'_>) -> ApiV3Result<OrderOutput> {
        let endpoint = format!("{}/api/v3/order", self.endpoint.as_ref());
        let query = OrderQuery::from(params);
        let signature = "";
        let query_with_signature = QueryWithSignature::new(query, signature);
        let response = self.reqwest_client.get(&endpoint).query(&query_with_signature).send().await?;
        let output = response.json::<OrderOutput>().await?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[tokio::test]
    async fn test_order() {
        let client = MexcApiClientWithAuthentication::new_for_test();
        let params = OrderParams {
            symbol: "KASUSDT",
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: Some(BigDecimal::from(1)),
            quote_order_quantity: None,
            price: Some(BigDecimal::from_str("0.00001").unwrap()),
            new_client_order_id: None,
        };
        let result = client.order(params).await;
        assert!(result.is_ok());
    }
}
