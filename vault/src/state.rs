use std::fmt::Display;

use bincode::{deserialize, serialize};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::ContractError;

#[cw_serde]
pub struct AddressesStruct {
    pub router: Addr,
    pub price_feed: Addr,
    pub usdg: Addr,
    pub error_controller: Addr,
}

#[cw_serde]
pub struct StateVariablesStruct {
    pub whitelisted_token_count: u128,

    pub max_leverage: u128,

    pub liquidation_fee_usd: u128,
    pub tax_basis_points: u128,
    pub stable_tax_basis_points: u128,
    pub mint_burn_fee_basis_points: u128,
    pub swap_fee_basis_points: u128,
    pub stable_swap_fee_basis_points: u128,
    pub margin_fee_basis_points: u128,

    pub min_profit_time: u128,
    pub has_dynamic_fees: bool,

    pub funding_interval: u64,
    pub funding_rate_factor: u128,
    pub stable_funding_rate_factor: u128,
    pub total_token_weights: u128,

    pub include_amm_price: bool,
    pub use_swap_pricing: bool,

    pub in_manager_mode: bool,
    pub in_private_liquidation_mode: bool,

    pub max_gas_price: u128,
    pub all_whitelisted_tokens: Vec<Addr>,
}

#[cw_serde]
pub struct Position {
    pub size: u128,
    pub collateral: u128,
    pub average_price: u128,
    pub entry_funding_rate: u128,
    pub reserve_amount: u128,
    pub realised_pnl: u128,
    pub last_increased_time: u128,
}
pub type Bytes<'a> = &'a [u8];

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct Key<T, U, V, W> {
    pub key_1: T,
    pub key_2: U,
    pub key_3: V,
    pub key_4: W,
}

impl<T, U, V, W> Key<T, U, V, W>
where
    T: Serialize + DeserializeOwned + Clone + Display + Sized,
    U: Serialize + DeserializeOwned + Clone + Display + Sized,
    V: Serialize + DeserializeOwned + Clone + Display + Sized,
    W: Serialize + DeserializeOwned + Clone + Display + Sized,
{
    pub fn new(key_1: T, key_2: U, key_3: V, key_4: W) -> Self {
        Self {
            key_1,
            key_2,
            key_3,
            key_4,
        }
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, ContractError> {
        match serialize(&self) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(ContractError::SerializationFailed {
                denom: self.key_1.to_string(),
                address: self.key_2.to_string(),
            }),
        }
    }

    pub fn as_bytes_std(&self) -> Result<Vec<u8>, StdError> {
        match serialize(&self) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(StdError::serialize_err(
                "Struct",
                format!("key_1: `{}`, key_2: `{}`!", self.key_1, self.key_2),
            )),
        }
    }

    pub fn from_bytes(&self, bytes: Bytes) -> Result<Self, ContractError> {
        match deserialize::<Self>(bytes) {
            Ok(key) => Ok(key),
            Err(_) => Err(ContractError::DeserializationFailed {}),
        }
    }
}

pub const GOV: Item<Addr> = Item::new("gov");

pub const ADDRESSES: Item<AddressesStruct> = Item::new("addresses");
pub const STATE_VARIABLES: Item<StateVariablesStruct> = Item::new("state_variables");

pub const IS_INITIALIZED: Item<bool> = Item::new("is_initialized"); // false at initialization
pub const IS_SWAP_ENABLED: Item<bool> = Item::new("isSwapEnabled"); // true at initialization
pub const IS_LEVERGE_ENABLED: Item<bool> = Item::new("isLeverageEnabled"); // true at initialization

// Mappings

// Nested Mapping not supported as in solidity so created 2 mapping for this
//     mapping (address => mapping (address => bool)) public override approvedRouters;
pub const APPROVED_ROUTERS1: Map<Addr, bool> = Map::new("approved_routers1");
pub const APPROVED_ROUTERS2: Map<Addr, bool> = Map::new("approved_routers1");

pub const IS_LIQUIDATOR: Map<Addr, bool> = Map::new("is_liquidator");
pub const IS_MANAGER: Map<Addr, bool> = Map::new("is_manager");

pub const WHITELISTED_TOKENS: Map<Addr, bool> = Map::new("whitelisted_tokens");
pub const TOKEN_DECIMALS: Map<Addr, u128> = Map::new("token_decimals");
pub const MIN_PROFIT_BASIS_POINTS: Map<Addr, u128> = Map::new("min_profit_basis_points");
pub const STABLE_TOKENS: Map<Addr, bool> = Map::new("stable_tokens");
pub const SHORTABLE_TOKENS: Map<Addr, bool> = Map::new("shortable_tokens");

// tokenBalances is used only to determine _transferIn values
pub const TOKEN_BALANCES: Map<Addr, u128> = Map::new("token_balancess");

// tokenWeights allows customisation of index composition
pub const TOKEN_WEIGHTS: Map<Addr, u128> = Map::new("token_weights");

// usdgAmounts tracks the amount of USDG debt for each whitelisted token
pub const USDG_AMOUNTS: Map<Addr, u128> = Map::new("usdg_Amounts");

// maxUsdgAmounts allows setting a max amount of USDG debt for a token
pub const MAX_USDG_AMOUNTS: Map<Addr, u128> = Map::new("max_usdg_Amounts");

// poolAmounts tracks the number of received tokens that can be used for leverage
// this is tracked separately from tokenBalances to exclude funds that are deposited as margin collateral
pub const POOL_AMOUNTS: Map<Addr, u128> = Map::new("pool_amounts");

// reservedAmounts tracks the number of tokens reserved for open leverage positions
pub const RSERVED_AMOUNTS: Map<Addr, u128> = Map::new("reserved_amounts");

// bufferAmounts allows specification of an amount to exclude from swaps
// this can be used to ensure a certain amount of liquidity is available for leverage positions
pub const BUFFER_AMOUNTS: Map<Addr, u128> = Map::new("buffer_amounts");

// guaranteedUsd tracks the amount of USD that is "guaranteed" by opened leverage positions
// this value is used to calculate the redemption values for selling of USDG
// this is an estimated amount, it is possible for the actual guaranteed value to be lower
// in the case of sudden price decreases, the guaranteed value should be corrected
// after liquidations are carried out
pub const GUARANTEED_USD: Map<Addr, u128> = Map::new("guaranteed_Usd");

// cumulativeFundingRates tracks the funding rates based on utilization
pub const COMMULATIVE_FUNDING_RATES: Map<Addr, u128> = Map::new("cumulative_Funding_Rates");

// lastFundingTimes tracks the last time funding was updated for a token
pub const LAST_FUNDING_TIMES: Map<Addr, u64> = Map::new("last_funding_times");

// positions tracks all open positions
pub const POSITIONS: Map<Bytes, Position> = Map::new("positions");

// feeReserves tracks the amount of fees per token
pub const FEE_RESERVES: Map<Addr, u128> = Map::new("fee_reserves");

pub const GLOBAL_SHORT_SIZES: Map<Addr, u128> = Map::new("global_Short_sizes");
pub const GLOBAL_SHORT_AVERAGE_PRIZES: Map<Addr, u128> = Map::new("global_short_average_sizes");
pub const MAX_GLOBAL_SHORT_SIZES: Map<Addr, u128> = Map::new("max_global_shoert_sizes");
pub const ERRORS: Map<u128, String> = Map::new("errors");
