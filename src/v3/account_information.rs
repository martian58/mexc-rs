use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crate::MexcApiClientWithAuthentication;
use crate::v3::{ApiResponse, ApiResult};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformationOutput {
    pub maker_commission: BigDecimal,
    pub taker_commission: BigDecimal,
    pub buyer_commission: BigDecimal,
    pub seller_commission: BigDecimal,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub update_time: Option<DateTime<Utc>>,
    pub account_type: String,
    pub balances: Vec<AccountBalance>,
    pub permissions: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub asset: String,
    pub free: BigDecimal,
    pub locked: BigDecimal,
}

#[async_trait]
pub trait AccountInformationEndpoint {
    async fn account_information(&self) -> ApiResult<AccountInformationOutput>;
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformationQuery {
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

#[async_trait]
impl AccountInformationEndpoint for MexcApiClientWithAuthentication {
    async fn account_information(&self) -> ApiResult<AccountInformationOutput> {
        let endpoint = format!("{}/api/v3/account", self.endpoint.as_ref());
        let query = self.sign_query(AccountInformationQuery {
            recv_window: None,
            timestamp: Utc::now(),
        })?;
        let response = self.reqwest_client.get(endpoint).query(&query).send().await?;
        let api_response = response.json::<ApiResponse<AccountInformationOutput>>().await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let client = MexcApiClientWithAuthentication::new_for_test();
        let result = client.account_information().await;
        assert!(result.is_ok());
    }
}
