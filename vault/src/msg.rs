use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Initialize {
        _router: Addr,
        _usdg: Addr,
        _price_feed: Addr,
        _liquidation_fee_usd: u128,
        _funding_rate_factor: u128,
        _stable_funding_rate_factor: u128,
    },
    SetErrorController {
        address: Addr,
    },
    SetError {
        error_code: u128,
        error: String,
    },
    SetMangerMode {
        in_manager_mode: bool,
    },
    SetManager {
        address: Addr,
        is_manager: bool,
    },
    SetInPrivateLiquidationMode {
        in_private_liquidation_mode: bool,
    },
    SetLiquidator {
        liquidator: Addr,
        is_active: bool,
    },
    SetIsSwapEnabaled {
        _is_swap_enabled: bool,
    },
    SetIsLevergaeEnabaled {
        _is_leverage_enabled: bool,
    },
    SetMaxGasPrice {
        max_gas_price: u128,
    },
    SetGov {
        gov: Addr,
    },
    SetPriceFeed {
        price_feed: Addr,
    },
    SetMaxLeverage {
        max_leverage: u128,
    },
    SetBufferAmount {
        token: Addr,
        amount: u128,
    },
    SetMaxGlobalShortSize {
        token: Addr,
        amount: u128,
    },
    SetFees {
        tax_basis_points: u128,
        stable_tax_basis_points: u128,
        mint_burn_fee_basis_points: u128,
        swap_fee_basis_points: u128,
        stable_swap_fee_basis_points: u128,
        margin_fee_basis_points: u128,
        liquidation_fee_usd: u128,
        min_profit_time: u128,
        has_dynamic_fees: bool,
    },
    SetFundingRate {
        funding_interval: u64,
        funding_rate_factor: u128,
        stable_funding_rate_factor: u128,
    },
    SetTokenConfig {
        token: Addr,
        token_decimals: u128,
        token_weight: u128,
        min_profit_bps: u128,
        max_usdg_amount: u128,
        is_stable: bool,
        is_shortable: bool,
    },

    ClearTokenConfig {
        token: Addr,
    },
    WithdrawFees {
        token: Addr,
        reciever: Addr,
    },
    AddRouters {
        router: Addr,
    },
    RemoveRouters {
        router: Addr,
    },
    SetUSDGAmount {
        token: Addr,
        amount: u128,
    },

    UpgradeVault {
        new_vault: Addr,
        token: Addr,
        amount: u128,
    },
    DirectPoolDeposit {
        token: Addr,
    },
    BuyUsdg {
        token: Addr,
        reciever: Addr,
    },
    SellUsdg {
        token: Addr,
        reciever: Addr,
    },
    Swap {
        token_in: Addr,
        token_out: Addr,
        reciever: Addr,
    },
    IncreasePosition {
        account: Addr,
        collateral_token: Addr,
        index_token: Addr,
        size_delta: u128,
        is_long: bool,
    },
    DecreasePosition {
        account: Addr,
        collateral_token: Addr,
        index_token: Addr,
        collateral_delta: u128,
        size_delta: u128,
        is_long: bool,
        reciever: Addr,
    },
    LiquidatePosition {
        account: Addr,
        collateral_token: Addr,
        index_token: Addr,
        is_long: bool,
        fee_reciever: Addr,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetRedemptionCollateral {
        token: Addr,
    },
    GetRedemptionCollateralUsd {
        token: Addr,
    },
    GetPosition {
        account: Addr,
        collateral_token: Addr,
        index_token: Addr,
        is_long: bool,
    },
    GetUtilisation {
        token: Addr,
    },
    GetPositionLeverage {
        account: Addr,
        collateral_token: Addr,
        index_token: Addr,
        is_long: bool,
    },
    GetGlobalShortDelta {
        token: Addr,
    },
    GetPositionDelta {
        account: Addr,
        collateral_token: Addr,
        index_token: Addr,
        is_long: bool,
    },
    GetTargetUsdgAmount {
        token: Addr,
    },
}

#[cw_serde]
pub struct GlobalShortDeltaResponse {
    pub has_profit: bool,
    pub delta: u128,
}
