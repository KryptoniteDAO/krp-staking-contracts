use cosmwasm_std::{
    Addr, Api, CosmosMsg, CustomQuery, Decimal, Querier, QuerierWrapper, QueryRequest, StdResult,
    Uint128,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub fn optional_addr_validate(api: &dyn Api, addr: Option<String>) -> StdResult<Option<Addr>> {
    let addr = if let Some(addr) = addr {
        Some(api.addr_validate(&addr)?)
    } else {
        None
    };

    Ok(addr)
}

/// QueryTaxWrapper is an override of QueryRequest::Custom for testing
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueryTaxWrapper {
    pub query_data: QueryTaxMsg,
}

impl From<QueryTaxWrapper> for CosmosMsg<QueryTaxWrapper> {
    fn from(s: QueryTaxWrapper) -> CosmosMsg<QueryTaxWrapper> {
        CosmosMsg::Custom(s)
    }
}

impl CustomQuery for QueryTaxWrapper {}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryTaxMsg {
    ///Query tax, usually is zero
    TaxRate {},
    ///Query tax cap, usually is zero
    TaxCap { denom: String },
}

/// TaxRateResponse is data format returned from TreasuryRequest::TaxRate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaxRateResponse {
    pub rate: Decimal,
}

/// TaxCapResponse is data format returned from TreasuryRequest::TaxCap query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaxCapResponse {
    pub cap: Uint128,
}

/// This is a helper wrapper to easily use our custom queries
pub struct CustomQuerier<'a, C: CustomQuery> {
    querier: QuerierWrapper<'a, C>,
}

impl<'a, C: CustomQuery> CustomQuerier<'a, C>
where
    QueryRequest<C>: From<QueryTaxWrapper>,
{
    pub fn new(query: &'a dyn Querier) -> Self {
        CustomQuerier {
            querier: QuerierWrapper::new(query),
        }
    }

    pub fn query_tax_cap<T: Into<String>>(&self, denom: T) -> StdResult<TaxCapResponse> {
        let request = QueryTaxWrapper {
            query_data: QueryTaxMsg::TaxCap {
                denom: denom.into(),
            },
        }
        .into();

        let res  = self.querier.query(&request);
        match res {
            Ok(_) => Ok(res.unwrap()),
            Err(_) => Ok({
                TaxCapResponse {
                    cap: Uint128::zero(),
                }
            }),
        }
    }

    pub fn query_tax_rate(&self) -> StdResult<TaxRateResponse> {
        let request = QueryTaxWrapper {
            query_data: QueryTaxMsg::TaxRate {},
        }
        .into();

        let res = self.querier.query(&request);
        match res {
            Ok(_) => Ok(res.unwrap()),
            Err(_) => Ok({
                TaxRateResponse {
                    rate: Decimal::zero(),
                }
            }),
        }
    }
}
