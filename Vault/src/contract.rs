use std::env;
use std::ops::{Add, Div, Mul, Sub};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GlobalShortDeltaResponse, InstantiateMsg, QueryMsg};
use crate::state::{
    AddressesStruct, Key, Position, StateVariablesStruct, ADDRESSES, APPROVED_ROUTERS1,
    APPROVED_ROUTERS2, BUFFER_AMOUNTS, COMMULATIVE_FUNDING_RATES, ERRORS, FEE_RESERVES,
    GLOBAL_SHORT_AVERAGE_PRIZES, GLOBAL_SHORT_SIZES, GOV, GUARANTEED_USD, IS_INITIALIZED,
    IS_LEVERGE_ENABLED, IS_LIQUIDATOR, IS_MANAGER, IS_SWAP_ENABLED, LAST_FUNDING_TIMES,
    MAX_GLOBAL_SHORT_SIZES, MAX_USDG_AMOUNTS, MIN_PROFIT_BASIS_POINTS, POOL_AMOUNTS, POSITIONS,
    RSERVED_AMOUNTS, SHORTABLE_TOKENS, STABLE_TOKENS, STATE_VARIABLES, TOKEN_BALANCES,
    TOKEN_DECIMALS, TOKEN_WEIGHTS, USDG_AMOUNTS, WHITELISTED_TOKENS,
};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, CosmosMsg, Deps, QuerierWrapper, StdError,
    StdResult, Storage, WasmQuery,
};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "vault.io:ft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const BASIS_POINTS_DIVISOR: u128 = 10000;
const FUNDING_RATE_PRECISION: u128 = 1000000;
const PRICE_PRECISION: u128 = 1000000000000000000000000;
const MIN_LEVERAGE: u128 = 10000; // 1x
const USDG_DECIMALS: u128 = 18;
const MAX_FEE_BASIS_POINTS: u128 = 500; // 5%
const MAX_LIQUIDATION_FEE_USD: u128 = 10000000000000000000000000000000; // 100 USD
const MIN_FUNDING_RATE_INTERVAL: u64 = 3600; //1 hour
const MAX_FUNDING_RATE_FACTOR: u128 = 10000; // 1%

// ********** Instantiate **********

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    GOV.save(deps.storage, &info.sender)?;
    IS_INITIALIZED.save(deps.storage, &false)?;
    IS_SWAP_ENABLED.save(deps.storage, &true)?;
    IS_LEVERGE_ENABLED.save(deps.storage, &true)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Initialize {
            _router,
            _usdg,
            _price_feed,
            _liquidation_fee_usd,
            _funding_rate_factor,
            _stable_funding_rate_factor,
        } => try_initialize(
            deps,
            info,
            env,
            _router,
            _usdg,
            _price_feed,
            _liquidation_fee_usd,
            _funding_rate_factor,
            _stable_funding_rate_factor,
        ),
        ExecuteMsg::SetError { error_code, error } => {
            try_set_error(deps, info, env, error_code, error)
        }
        ExecuteMsg::SetErrorController { address } => {
            try_set_error_controller(deps, info, env, address)
        }
        ExecuteMsg::SetMangerMode { in_manager_mode } => {
            try_set_in_managermode(deps, info, env, in_manager_mode)
        }
        ExecuteMsg::SetManager {
            address,
            is_manager,
        } => try_set_manager(deps, info, env, address, is_manager),
        ExecuteMsg::SetInPrivateLiquidationMode {
            in_private_liquidation_mode,
        } => try_set_in_private_liquidation_mode(deps, info, env, in_private_liquidation_mode),
        ExecuteMsg::SetLiquidator {
            liquidator,
            is_active,
        } => try_set_liquidator(deps, info, env, liquidator, is_active),
        ExecuteMsg::SetIsSwapEnabaled { _is_swap_enabled } => {
            try_set_is_swap_enabled(deps, info, env, _is_swap_enabled)
        }
        ExecuteMsg::SetIsLevergaeEnabaled {
            _is_leverage_enabled,
        } => try_set_is_leverage_enabled(deps, info, env, _is_leverage_enabled),
        ExecuteMsg::SetMaxGasPrice { max_gas_price } => {
            try_set_max_gas_price(deps, info, env, max_gas_price)
        }
        ExecuteMsg::SetGov { gov } => try_set_gov(deps, info, env, gov),
        ExecuteMsg::SetPriceFeed { price_feed } => try_set_price_feed(deps, info, env, price_feed),
        ExecuteMsg::SetMaxLeverage { max_leverage } => {
            try_set_max_leverage(deps, info, env, max_leverage)
        }
        ExecuteMsg::SetBufferAmount { token, amount } => {
            try_set_buffer_amount(deps, info, env, token, amount)
        }
        ExecuteMsg::SetMaxGlobalShortSize { token, amount } => {
            try_set_max_global_short_size(deps, info, env, token, amount)
        }
        ExecuteMsg::SetFees {
            tax_basis_points,
            stable_tax_basis_points,
            mint_burn_fee_basis_points,
            swap_fee_basis_points,
            stable_swap_fee_basis_points,
            margin_fee_basis_points,
            liquidation_fee_usd,
            min_profit_time,
            has_dynamic_fees,
        } => try_set_fees(
            deps,
            info,
            env,
            tax_basis_points,
            stable_tax_basis_points,
            mint_burn_fee_basis_points,
            swap_fee_basis_points,
            stable_swap_fee_basis_points,
            margin_fee_basis_points,
            liquidation_fee_usd,
            min_profit_time,
            has_dynamic_fees,
        ),
        ExecuteMsg::SetFundingRate {
            funding_interval,
            funding_rate_factor,
            stable_funding_rate_factor,
        } => try_set_funding_rate(
            deps,
            info,
            env,
            funding_interval,
            funding_rate_factor,
            stable_funding_rate_factor,
        ),
        ExecuteMsg::SetTokenConfig {
            token,
            token_decimals,
            token_weight,
            min_profit_bps,
            max_usdg_amount,
            is_stable,
            is_shortable,
        } => try_set_token_config(
            deps,
            info,
            env,
            token,
            token_decimals,
            token_weight,
            min_profit_bps,
            max_usdg_amount,
            is_stable,
            is_shortable,
        ),
        ExecuteMsg::ClearTokenConfig { token } => try_clear_token_config(deps, info, env, token),
        ExecuteMsg::WithdrawFees { token, reciever } => {
            try_withdraw_fees(deps, info, env, token, reciever)
        }
        ExecuteMsg::AddRouters { router } => try_add_routers(deps, info, env, router),
        ExecuteMsg::RemoveRouters { router } => try_remove_routers(deps, info, env, router),
        ExecuteMsg::SetUSDGAmount { token, amount } => {
            try_set_usdg_amount(deps, info, env, token, amount)
        }

        ExecuteMsg::UpgradeVault {
            new_vault,
            token,
            amount,
        } => try_upgrade_vault(deps, info, env, new_vault, token, amount),
        ExecuteMsg::DirectPoolDeposit { token } => try_direct_pool_deposit(deps, info, env, token),
        ExecuteMsg::BuyUsdg { token, reciever } => try_buy_usdg(deps, info, env, token, reciever),
        ExecuteMsg::SellUsdg { token, reciever } => try_sell_usdg(deps, info, env, token, reciever),
        ExecuteMsg::Swap {
            token_in,
            token_out,
            reciever,
        } => try_swap(deps, info, env, token_in, token_out, reciever),
        ExecuteMsg::IncreasePosition {
            account,
            collateral_token,
            index_token,
            size_delta,
            is_long,
        } => try_increase_position(
            deps,
            info,
            env,
            account,
            collateral_token,
            index_token,
            size_delta,
            is_long,
        ),
        ExecuteMsg::DecreasePosition {
            account,
            collateral_token,
            index_token,
            collateral_delta,
            size_delta,
            is_long,
            reciever,
        } => try_decrease_position(
            deps,
            info,
            env,
            account,
            collateral_token,
            index_token,
            collateral_delta,
            size_delta,
            is_long,
            reciever,
        ),

        ExecuteMsg::LiquidatePosition {
            account,
            collateral_token,
            index_token,
            is_long,
            fee_reciever,
        } => try_liquidate_position(
            deps,
            info,
            env,
            account,
            collateral_token,
            index_token,
            is_long,
            fee_reciever,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: DepsMut, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetRedemptionCollateral { token } => try_get_redemption_collateral(deps, token),
        QueryMsg::GetRedemptionCollateralUsd { token } => {
            try_get_redemption_collateral_usd(deps, token)
        }
        QueryMsg::GetPosition {
            account,
            collateral_token,
            index_token,
            is_long,
        } => try_get_position(deps, account, collateral_token, index_token, is_long),
        QueryMsg::GetUtilisation { token } => try_get_utilisation(deps, token),
        QueryMsg::GetPositionLeverage {
            account,
            collateral_token,
            index_token,
            is_long,
        } => try_get_position_leverage(deps, account, collateral_token, index_token, is_long),
        QueryMsg::GetGlobalShortDelta { token } => try_global_short_delta(deps, token),
        QueryMsg::GetPositionDelta {
            account,
            collateral_token,
            index_token,
            is_long,
        } => try_get_position_delta(deps, env, account, collateral_token, index_token, is_long),
        QueryMsg::GetTargetUsdgAmount { token } => try_get_target_usdg_amount(deps, env, token),
    }
}

// ********** Transactions **********

fn try_initialize(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _router: Addr,
    _usdg: Addr,
    _price_feed: Addr,
    _liquidation_fee_usd: u128,
    _funding_rate_factor: u128,
    _stable_funding_rate_factor: u128,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);
    let mut is_initialized = IS_INITIALIZED.load(deps.storage)?;
    _validate(!is_initialized, 1)?;
    is_initialized = true;
    IS_INITIALIZED.save(deps.storage, &is_initialized)?;

    let state_variable = StateVariablesStruct {
        whitelisted_token_count: 0,
        max_leverage: 50 * 10000, // 50x
        liquidation_fee_usd: _liquidation_fee_usd,
        tax_basis_points: 50,
        stable_tax_basis_points: 20,
        mint_burn_fee_basis_points: 30,
        swap_fee_basis_points: 30,
        stable_swap_fee_basis_points: 4,
        margin_fee_basis_points: 10,

        min_profit_time: 0,
        has_dynamic_fees: false,
        funding_interval: 8 * 3600,
        funding_rate_factor: _funding_rate_factor,
        stable_funding_rate_factor: _stable_funding_rate_factor,
        total_token_weights: 0,
        include_amm_price: true,
        use_swap_pricing: false,
        in_manager_mode: false,
        in_private_liquidation_mode: false,
        max_gas_price: 0,
        all_whitelisted_tokens: Vec::new(),
    };

    let addresses = AddressesStruct {
        router: _router,
        price_feed: _price_feed,
        usdg: _usdg,
        error_controller: Addr::unchecked(""),
    };
    ADDRESSES.save(deps.storage, &addresses)?;

    STATE_VARIABLES.save(deps.storage, &state_variable)?;

    Ok(Response::new().add_attribute("method", "initialize"))
}

fn try_set_error_controller(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    addr: Addr,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);
    let mut addresses = ADDRESSES.load(deps.storage)?;
    addresses.error_controller = addr;
    ADDRESSES.save(deps.storage, &addresses)?;

    Ok(Response::new().add_attribute("method", "set_error_controller"))
}

fn try_set_error(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _error_code: u128,
    error: String,
) -> Result<Response, ContractError> {
    let addresses = ADDRESSES.load(deps.storage)?;

    if info.sender != addresses.error_controller {
        return Err(ContractError::CustomError {
            val: "Vault: Invalid error controller".to_string(),
        });
    }
    ERRORS.save(deps.storage, _error_code, &error)?;
    Ok(Response::new().add_attribute("method", "set_errorr"))
}

fn try_set_in_managermode(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    in_managerode: bool,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    state_variables.in_manager_mode = in_managerode;
    STATE_VARIABLES.save(deps.storage, &state_variables)?;

    Ok(Response::new().add_attribute("method", "set_in_manager_mode"))
}

fn try_set_manager(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _manager: Addr,
    _is_manager: bool,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut is_manager = IS_MANAGER
        .load(deps.storage, _manager.clone())
        .unwrap_or_default();
    is_manager = _is_manager;
    IS_MANAGER.save(deps.storage, _manager, &is_manager)?;

    Ok(Response::new().add_attribute("method", "set_manager"))
}

fn try_set_in_private_liquidation_mode(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    in_private_liquidation_mode: bool,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    state_variables.in_private_liquidation_mode = in_private_liquidation_mode;
    STATE_VARIABLES.save(deps.storage, &state_variables)?;
    Ok(Response::new().add_attribute("method", "set_in_private_liquidation_mode"))
}

fn try_set_liquidator(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _liquidator: Addr,
    _is_liquidator: bool,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut is_liquidator = IS_LIQUIDATOR
        .load(deps.storage, _liquidator.clone())
        .unwrap_or_default();
    is_liquidator = _is_liquidator;
    IS_LIQUIDATOR.save(deps.storage, _liquidator, &is_liquidator)?;
    Ok(Response::new().add_attribute("method", "set_liquidator"))
}

fn try_set_is_swap_enabled(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _is_swap_enabled: bool,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut is_swap_enabled = IS_SWAP_ENABLED.load(deps.storage)?;
    is_swap_enabled = _is_swap_enabled;
    IS_SWAP_ENABLED.save(deps.storage, &is_swap_enabled)?;

    Ok(Response::new().add_attribute("method", "set_is_swap_enabled"))
}

fn try_set_is_leverage_enabled(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _is_leverage_enabled: bool,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut is_leverage_enabled = IS_LEVERGE_ENABLED.load(deps.storage)?;
    is_leverage_enabled = _is_leverage_enabled;
    IS_LEVERGE_ENABLED.save(deps.storage, &is_leverage_enabled)?;

    Ok(Response::new().add_attribute("method", "set_is_leverage_enabled"))
}

fn try_set_max_gas_price(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _max_gas_price: u128,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut state_variables: StateVariablesStruct = STATE_VARIABLES.load(deps.storage)?;
    state_variables.max_gas_price = _max_gas_price;
    STATE_VARIABLES.save(deps.storage, &state_variables)?;

    Ok(Response::new().add_attribute("method", "set_max_gas_price"))
}

fn try_set_gov(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _gov: Addr,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut gov = GOV.load(deps.storage)?;
    gov = _gov;
    GOV.save(deps.storage, &gov)?;

    Ok(Response::new().add_attribute("method", "set_gov"))
}

fn try_set_price_feed(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _price_feed: Addr,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut addresses = ADDRESSES.load(deps.storage)?;
    addresses.price_feed = _price_feed;
    ADDRESSES.save(deps.storage, &addresses)?;

    Ok(Response::new().add_attribute("method", "set_price_feed"))
}

fn try_set_max_leverage(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _max_leverage: u128,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    _validate(_max_leverage > MIN_LEVERAGE, 2)?;

    let mut state_variables: StateVariablesStruct = STATE_VARIABLES.load(deps.storage)?;
    state_variables.max_leverage = _max_leverage;
    STATE_VARIABLES.save(deps.storage, &state_variables)?;

    Ok(Response::new().add_attribute("method", "set_max_leverage"))
}

fn try_set_buffer_amount(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _token: Addr,
    _amount: u128,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut buffer_amounts = BUFFER_AMOUNTS
        .load(deps.storage, _token.clone())
        .unwrap_or_default();
    buffer_amounts = _amount;
    BUFFER_AMOUNTS.save(deps.storage, _token, &buffer_amounts)?;

    Ok(Response::new().add_attribute("method", "set_buffer_amount"))
}

fn try_set_max_global_short_size(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _token: Addr,
    _amount: u128,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut max_global_short_sizes: u128 = MAX_GLOBAL_SHORT_SIZES
        .load(deps.storage, _token.clone())
        .unwrap_or_default();
    max_global_short_sizes = _amount;
    MAX_GLOBAL_SHORT_SIZES.save(deps.storage, _token, &&max_global_short_sizes)?;

    Ok(Response::new().add_attribute("method", "set_max_global_short_size"))
}

fn try_set_fees(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _tax_basis_points: u128,
    _stable_tax_basis_points: u128,
    _mint_burn_fee_basis_points: u128,
    _swap_fee_basis_points: u128,
    _stable_swap_fee_basis_points: u128,
    _margin_fee_basis_points: u128,
    _liquidation_fee_usd: u128,
    _min_profit_time: u128,
    _has_dynamic_fees: bool,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);
    _validate(_tax_basis_points <= MAX_FEE_BASIS_POINTS, 3)?;
    _validate(_stable_tax_basis_points <= MAX_FEE_BASIS_POINTS, 4)?;
    _validate(_mint_burn_fee_basis_points <= MAX_FEE_BASIS_POINTS, 5)?;
    _validate(_swap_fee_basis_points <= MAX_FEE_BASIS_POINTS, 6)?;
    _validate(_stable_swap_fee_basis_points <= MAX_FEE_BASIS_POINTS, 7)?;
    _validate(_margin_fee_basis_points <= MAX_FEE_BASIS_POINTS, 8)?;
    _validate(_liquidation_fee_usd <= MAX_FEE_BASIS_POINTS, 9)?;

    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    state_variables.tax_basis_points = _tax_basis_points;
    state_variables.stable_tax_basis_points = _stable_tax_basis_points;
    state_variables.mint_burn_fee_basis_points = _mint_burn_fee_basis_points;
    state_variables.swap_fee_basis_points = _swap_fee_basis_points;
    state_variables.stable_swap_fee_basis_points = _stable_swap_fee_basis_points;
    state_variables.margin_fee_basis_points = _margin_fee_basis_points;
    state_variables.liquidation_fee_usd = _liquidation_fee_usd;
    state_variables.min_profit_time = _min_profit_time;
    state_variables.has_dynamic_fees = _has_dynamic_fees;

    STATE_VARIABLES.save(deps.storage, &state_variables)?;

    Ok(Response::new().add_attribute("method", "set_fees"))
}

fn try_set_funding_rate(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _funding_interval: u64,
    _funding_rate_factor: u128,
    _stable_funding_rate_factor: u128,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);
    _validate(_funding_interval <= MIN_FUNDING_RATE_INTERVAL, 10)?;
    _validate(_funding_rate_factor <= MAX_FUNDING_RATE_FACTOR, 11)?;
    _validate(_stable_funding_rate_factor <= MAX_FUNDING_RATE_FACTOR, 12)?;

    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    state_variables.funding_interval = _funding_interval;
    state_variables.funding_rate_factor = _funding_rate_factor;
    state_variables.stable_funding_rate_factor = _stable_funding_rate_factor;

    STATE_VARIABLES.save(deps.storage, &state_variables)?;

    Ok(Response::new().add_attribute("method", "set_funding_rate"))
}

fn try_set_token_config(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _token: Addr,
    _token_decimals: u128,
    _token_weight: u128,
    _min_profit_bps: u128,
    _max_usdg_amount: u128,
    _is_stable: bool,
    _is_Shortable: bool,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let mut whitelistedtoken = WHITELISTED_TOKENS.load(deps.storage, _token.clone())?;
    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    let mut token_weights = TOKEN_WEIGHTS.load(deps.storage, _token.clone())?;
    let mut token_decimals = TOKEN_DECIMALS.load(deps.storage, _token.clone())?;
    let mut min_profit_basis_points = MIN_PROFIT_BASIS_POINTS.load(deps.storage, _token.clone())?;
    let mut max_usdg_amounts = MAX_USDG_AMOUNTS.load(deps.storage, _token.clone())?;
    let mut stable_tokens = STABLE_TOKENS.load(deps.storage, _token.clone())?;
    let mut shortable_tokens = SHORTABLE_TOKENS.load(deps.storage, _token.clone())?;

    // increment token count for the first time
    if !whitelistedtoken {
        state_variables.whitelisted_token_count += 1;
        state_variables.all_whitelisted_tokens.push(_token.clone());
    }
    let mut _total_token_weights = state_variables.total_token_weights;
    _total_token_weights = _total_token_weights.sub(token_weights);

    whitelistedtoken = true;
    token_decimals = _token_decimals;
    token_weights = _token_weight;
    min_profit_basis_points = _min_profit_bps;
    max_usdg_amounts = _max_usdg_amount;
    stable_tokens = _is_stable;
    shortable_tokens = _is_Shortable;

    state_variables.total_token_weights = _total_token_weights.add(_token_weight);
    WHITELISTED_TOKENS.save(deps.storage, _token.clone(), &whitelistedtoken)?;
    STATE_VARIABLES.save(deps.storage, &state_variables)?;
    TOKEN_WEIGHTS.save(deps.storage, _token.clone(), &token_weights)?;
    TOKEN_DECIMALS.save(deps.storage, _token.clone(), &token_decimals)?;
    MIN_PROFIT_BASIS_POINTS.save(deps.storage, _token.clone(), &min_profit_basis_points)?;
    MAX_USDG_AMOUNTS.save(deps.storage, _token.clone(), &min_profit_basis_points)?;
    STABLE_TOKENS.save(deps.storage, _token.clone(), &stable_tokens)?;
    SHORTABLE_TOKENS.save(deps.storage, _token.clone(), &shortable_tokens)?;

    // Validate price feed
    // using hardcoded value as in cosmwasm there was no IvaultPricefeed interface avialable

    get_max_price(_token);

    Ok(Response::new().add_attribute("method", "set_token_config"))
}

fn try_clear_token_config(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _token: Addr,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);
    let whitelistedtoken = WHITELISTED_TOKENS.load(deps.storage, _token.clone())?;

    _validate(whitelistedtoken, 13)?;

    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    let token_weights = TOKEN_WEIGHTS.load(deps.storage, _token.clone())?;

    state_variables.total_token_weights = state_variables.total_token_weights.sub(token_weights);
    WHITELISTED_TOKENS.remove(deps.storage, _token.clone());
    TOKEN_DECIMALS.remove(deps.storage, _token.clone());
    TOKEN_WEIGHTS.remove(deps.storage, _token.clone());
    MIN_PROFIT_BASIS_POINTS.remove(deps.storage, _token.clone());
    MAX_USDG_AMOUNTS.remove(deps.storage, _token.clone());
    STABLE_TOKENS.remove(deps.storage, _token.clone());
    SHORTABLE_TOKENS.remove(deps.storage, _token.clone());

    STATE_VARIABLES.save(deps.storage, &state_variables)?;

    Ok(Response::new().add_attribute("method", "clear_token_config"))
}

fn try_withdraw_fees(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _token: Addr,
    _reciever: Addr,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender.clone());

    let mut fee_reserves = FEE_RESERVES.load(deps.storage, _token.clone())?;

    let amount = fee_reserves;

    if amount == 0 {
        return Ok(Response::new()
            .add_attribute("method", "withdraw_fees")
            .add_attribute("amount", 0.to_string()));
    }
    fee_reserves = 0;

    _transfer_out(info, _token.clone(), amount, _reciever)?;
    FEE_RESERVES.save(deps.storage, _token, &fee_reserves)?;

    Ok(Response::new()
        .add_attribute("method", "withdraw_fees")
        .add_attribute("amount", amount.to_string()))
}

fn try_add_routers(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _router: Addr,
) -> Result<Response, ContractError> {
    let mut approved_routers1 = APPROVED_ROUTERS1.load(deps.storage, _router.clone())?;
    let mut approved_routers2 = APPROVED_ROUTERS2.load(deps.storage, _router)?;
    approved_routers1 = true;
    approved_routers2 = approved_routers1;

    APPROVED_ROUTERS1.save(deps.storage, info.sender.clone(), &approved_routers1)?;
    APPROVED_ROUTERS2.save(deps.storage, info.sender, &approved_routers2)?;

    Ok(Response::new().add_attribute("method", "add_routers"))
}

fn try_remove_routers(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _router: Addr,
) -> Result<Response, ContractError> {
    let mut approved_routers1 = APPROVED_ROUTERS1.load(deps.storage, _router.clone())?;
    let mut approved_routers2 = APPROVED_ROUTERS2.load(deps.storage, _router)?;
    approved_routers1 = false;
    approved_routers2 = approved_routers1;

    APPROVED_ROUTERS1.save(deps.storage, info.sender.clone(), &approved_routers1)?;
    APPROVED_ROUTERS2.save(deps.storage, info.sender, &approved_routers2)?;

    Ok(Response::new().add_attribute("method", "remove_routers"))
}

fn try_set_usdg_amount(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _token: Addr,
    _amount: u128,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let usdg_amount = USDG_AMOUNTS.load(deps.storage, _token.clone())?;

    if _amount > usdg_amount {
        _increase_usdg_amount(deps.storage, _token.clone(), _amount)?;
        return Ok(Response::new().add_attribute("method", "set_usdg_Amount"));
    }

    _decrease_usdg_amount(deps.storage, _token, _amount)?;

    Ok(Response::new().add_attribute("method", "set_usdg_Amount"))
}

// the governance controlling this function should have a timelock
fn try_upgrade_vault(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    _new_vault: Addr,
    _token: Addr,
    _amount: u128,
) -> Result<Response, ContractError> {
    only_gov(deps.storage, info.sender);

    let transfer_from_msg = cw20::Cw20ExecuteMsg::Transfer {
        recipient: _new_vault.to_string(),
        amount: _amount.into(),
    };
    let msg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: _token.to_string(),
        msg: to_binary(&transfer_from_msg)?,
        funds: info.funds,
    });

    Ok(Response::new()
        .add_attribute("method", "upgrade_vault")
        .add_message(msg))
}

// deposit into the pool without minting USDG tokens
// useful in allowing the pool to become over-collaterised
fn try_direct_pool_deposit(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    _token: Addr,
) -> Result<Response, ContractError> {
    let whitelisted_tokens = WHITELISTED_TOKENS.load(deps.storage, _token.clone())?;
    _validate(whitelisted_tokens, 14)?;

    let token_amount = _transfer_in(&deps, env.clone(), info, _token.clone())?;
    _validate(token_amount > 0, 15)?;
    increase_pool_amount(deps.storage, deps.querier, env, _token, token_amount)?;

    Ok(Response::new().add_attribute("method", "direct_pool_deposit"))
}

fn try_buy_usdg(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    _token: Addr,
    _reciever: Addr,
) -> Result<Response, ContractError> {
    _validate_manager(deps.storage, info.sender.clone());
    let whitelisted_tokens = WHITELISTED_TOKENS.load(deps.storage, _token.clone())?;
    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    _validate(whitelisted_tokens, 16)?;
    state_variables.use_swap_pricing = true;

    let token_amount = _transfer_in(&deps, env.clone(), info, _token.clone())?;
    _validate(token_amount > 0, 17)?;

    update_cumulative_funding_rate(deps.storage, env.clone(), _token.clone(), _token.clone())?;

    let price = 0; // getMinPrice(_token); uses Ivault interface so using hardcode value for the task

    let usdg_amount = token_amount.mul(price).div(PRICE_PRECISION);
    let addresses = ADDRESSES.load(deps.storage)?;

    let usdg_amount = adjust_for_decimals(
        deps.storage,
        usdg_amount,
        _token.clone(),
        addresses.usdg.clone(),
    )?;
    _validate(usdg_amount > 0, 18)?;

    let fee_basis_points = 0; // vaultUtils.getBuyUsdgFeeBasisPoints(_token, usdgAmount); uses VaultUtils interface so using hardcode value for the task
    let amount_after_fees =
        _collect_swap_fees(deps.storage, _token.clone(), token_amount, fee_basis_points)?;
    let mut mint_amount = amount_after_fees.mul(price).div(PRICE_PRECISION);
    mint_amount = adjust_for_decimals(deps.storage, mint_amount, _token.clone(), addresses.usdg)?;

    _increase_usdg_amount(deps.storage, _token.clone(), mint_amount)?;
    state_variables.use_swap_pricing = false;
    STATE_VARIABLES.save(deps.storage, &state_variables)?;
    _increase_pool_amount(
        deps.storage,
        deps.querier,
        env,
        _token.clone(),
        amount_after_fees,
    )?;

    // IUSDG(usdg).mint(_receiver, mintAmount); // don.t have this function so it could have been a cw20 token function but haven't added

    Ok(Response::new()
        .add_attribute("method", "buy_usdg")
        .add_attribute("reciever", _reciever)
        .add_attribute("token", _token.to_string())
        .add_attribute("token_amount", token_amount.to_string())
        .add_attribute("mint_amount", mint_amount.to_string())
        .add_attribute("fee_basis_points", fee_basis_points.to_string()))
}

fn try_sell_usdg(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    _token: Addr,
    _reciever: Addr,
) -> Result<Response, ContractError> {
    _validate_manager(deps.storage, info.sender.clone());
    let whitelisted_tokens = WHITELISTED_TOKENS.load(deps.storage, _token.clone())?;
    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    let addresses = ADDRESSES.load(deps.storage)?;
    _validate(whitelisted_tokens, 19)?;
    state_variables.use_swap_pricing = true;

    let usdg_amount = _transfer_in(&deps, env.clone(), info.clone(), addresses.usdg.clone())?;
    _validate(usdg_amount > 0, 20)?;
    update_cumulative_funding_rate(deps.storage, env.clone(), _token.clone(), _token.clone())?;
    let redemption_amount =
        get_redemption_amount(deps.storage, env.clone(), _token.clone(), usdg_amount)?;
    _validate(redemption_amount > 0, 21)?;
    _decrease_usdg_amount(deps.storage, _token.clone(), usdg_amount)?;
    _decrease_pool_amount(deps.storage, env.clone(), _token.clone(), redemption_amount)?;

    // IUSDG(usdg).burn(address(this), usdgAmount);  // don.t have this function so it could have been a cw20 token function but haven't added

    // the _transferIn call increased the value of tokenBalances[usdg]
    // usually decreases in token balances are synced by calling _transferOut
    // however, for usdg, the tokens are burnt, so _updateTokenBalance should
    // be manually called to record the decrease in tokens

    let fee_basis_points = 0; // Used hardcoded as no IVaultUtils present in cosmwasm
    let amount_out = _collect_swap_fees(
        deps.storage,
        _token.clone(),
        redemption_amount,
        fee_basis_points,
    )?;
    _validate(amount_out > 0, 22)?;
    _transfer_out(info, _token.clone(), amount_out, _reciever.clone())?;

    state_variables.use_swap_pricing = false;
    STATE_VARIABLES.save(deps.storage, &state_variables)?;

    _update_token_balance(deps.storage, deps.querier, env, addresses.usdg)?;

    Ok(Response::new()
        .add_attribute("method", "sell_usdg")
        .add_attribute("reciever", _reciever.to_string())
        .add_attribute("token", _token.to_string())
        .add_attribute("usdg_amount", usdg_amount.to_string())
        .add_attribute("amount_out", amount_out.to_string())
        .add_attribute("fee_basis_points", fee_basis_points.to_string()))
}

fn try_swap(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    _token_in: Addr,
    _token_out: Addr,
    _reciever: Addr,
) -> Result<Response, ContractError> {
    let is_swap_enabled = IS_SWAP_ENABLED.load(deps.storage)?;
    let whitelisted_tokens_in = WHITELISTED_TOKENS.load(deps.storage, _token_in.clone())?;
    let whitelisted_tokens_out = WHITELISTED_TOKENS.load(deps.storage, _token_out.clone())?;
    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    let addresses = ADDRESSES.load(deps.storage)?;

    _validate(is_swap_enabled, 23)?;
    _validate(whitelisted_tokens_in, 24)?;
    _validate(whitelisted_tokens_out, 25)?;

    state_variables.use_swap_pricing = true;

    update_cumulative_funding_rate(
        deps.storage,
        env.clone(),
        _token_in.clone(),
        _token_in.clone(),
    )?;
    update_cumulative_funding_rate(
        deps.storage,
        env.clone(),
        _token_out.clone(),
        _token_out.clone(),
    )?;

    let amount_in = _transfer_in(&deps, env.clone(), info.clone(), _token_in.clone())?;
    _validate(amount_in > 0, 27)?;

    let price_in = 0; // Hardcode as IVaultPriceFeed(priceFeed).getPrice(_token, false, includeAmmPrice, useSwapPricing); is not there in cosmwasm
    let price_out = get_max_price(_token_out.clone());

    let mut amount_out = amount_in.mul(price_in).div(PRICE_PRECISION);

    amount_out = adjust_for_decimals(
        deps.storage,
        amount_out,
        _token_in.clone(),
        _token_out.clone(),
    )?;

    // adjust usdgAmounts by the same usdgAmount as debt is shifted between the assets
    let mut usdg_amount = amount_in.mul(price_in).div(PRICE_PRECISION);
    usdg_amount =
        adjust_for_decimals(deps.storage, usdg_amount, _token_in.clone(), addresses.usdg)?;

    let fee_basis_points = 0; //hardcoded as vaultUtils.getSwapFeeBasisPoints(_tokenIn, _tokenOut, usdgAmount); is not present is cosmwasm

    let amount_out_after_fees = _collect_swap_fees(
        deps.storage,
        _token_in.clone(),
        amount_out,
        fee_basis_points,
    )?;

    _increase_usdg_amount(deps.storage, _token_in.clone(), usdg_amount)?;
    _decrease_usdg_amount(deps.storage, _token_out.clone(), usdg_amount)?;

    _decrease_pool_amount(deps.storage, env.clone(), _token_out.clone(), amount_out)?;
    _validate_buffer_amount(deps.storage, _token_out.clone())?;

    _transfer_out(
        info.clone(),
        _token_out.clone(),
        amount_out_after_fees,
        _reciever.clone(),
    )?;

    state_variables.use_swap_pricing = false;

    STATE_VARIABLES.load(deps.storage)?;
    _increase_pool_amount(
        deps.storage,
        deps.querier,
        env.clone(),
        _token_in.clone(),
        amount_in,
    )?;

    Ok(Response::new()
        .add_attribute("method", "swap")
        .add_attribute("reciever", _reciever.to_string())
        .add_attribute("token_in", _token_in.to_string())
        .add_attribute("token_out", _token_out.to_string())
        .add_attribute("amount_in", amount_in.to_string())
        .add_attribute("amount_out", amount_out.to_string())
        .add_attribute("amount__out_after_fees", amount_out_after_fees.to_string())
        .add_attribute("fee_basis_points", fee_basis_points.to_string()))
}

fn try_increase_position(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    size_delta: u128,
    is_long: bool,
) -> Result<Response, ContractError> {
    let is_leverage_enabled = IS_LEVERGE_ENABLED.load(deps.storage)?;
    let global_short_sizes = GLOBAL_SHORT_SIZES.load(deps.storage, index_token.clone())?;
    let mut global_short_average_prizes =
        GLOBAL_SHORT_AVERAGE_PRIZES.load(deps.storage, index_token.clone())?;

    _validate(is_leverage_enabled, 28)?;
    _validate_gas_price(deps.storage, env.clone());
    _validate_router(deps.storage, info.clone(), account.clone());
    validate_tokens(
        deps.storage,
        collateral_token.clone(),
        index_token.clone(),
        is_long.clone(),
    );

    // vaultUtils.validateIncreasePosition(_account, _collateralToken, _indexToken, _sizeDelta, _isLong); //not present in cosmwasm

    update_cumulative_funding_rate(
        deps.storage,
        env.clone(),
        collateral_token.clone(),
        index_token.clone(),
    )?;

    let key = get_position_key(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long.clone(),
    )?;

    let mut positions = POSITIONS.load(deps.storage, &key)?;
    let mut price = 0;
    if is_long {
        price = get_max_price(index_token.clone());
    } else {
        price = 0; // harcoded as IVaultPriceFeed(priceFeed).getPrice(_token, false, includeAmmPrice, useSwapPricing); is not present in cosmwasm
    }

    if positions.size == 0 {
        positions.average_price = price;
    }

    if positions.size > 0 && size_delta > 0 {
        positions.average_price = get_next_average_price(
            index_token.clone(),
            positions.size,
            positions.average_price,
            is_long,
            price,
            size_delta,
            positions.last_increased_time,
            env.clone(),
            deps.storage,
        )?;
    }

    let fee = collect_margin_fees(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long,
        size_delta,
        positions.size,
        positions.entry_funding_rate,
        deps.storage,
    )?;

    let collateral_delta =
        _transfer_in(&deps, env.clone(), info.clone(), collateral_token.clone())?;
    let collateral_delta_usd =
        token_to_usd_min(collateral_token.clone(), collateral_delta, deps.storage)?;

    positions.collateral.add(collateral_delta_usd);
    _validate(positions.collateral >= fee, 29)?;
    positions.collateral.sub(fee);
    positions.entry_funding_rate = 0; // Hardcoded
    positions.size.add(size_delta);
    positions.last_increased_time = env.block.time.seconds() as u128;
    _validate(positions.size > 0, 30)?;
    validate_position(positions.size, positions.collateral);
    // validateLiquidation(_account, _collateralToken, _indexToken, _isLong, true); // not present in cosmwasm

    // reserve tokens to pay profits on the position
    let reserved_delta = usd_to_token_min(
        collateral_token.clone(),
        positions.reserve_amount,
        deps.storage,
    )?;
    positions.reserve_amount.add(reserved_delta);

    increase_reserved_amount(deps.storage, collateral_token.clone(), reserved_delta)?;

    if is_long {
        // guaranteedUsd stores the sum of (position.size - position.collateral) for all positions
        // if a fee is charged on the collateral then guaranteedUsd should be increased by that fee amount
        // since (position.size - position.collateral) would have increased by `fee`
        increase_guarnteed_usd(deps.storage, collateral_token.clone(), size_delta.add(fee))?;
        decrease_guarnteed_usd(deps.storage, collateral_token.clone(), collateral_delta_usd)?;

        // treat the deposited collateral as part of the pool
        increase_pool_amount(
            deps.storage,
            deps.querier,
            env.clone(),
            collateral_token.clone(),
            collateral_delta,
        )?;
        // fees need to be deducted from the pool since fees are deducted from position.collateral
        // and collateral is treated as part of the pool
        let amount = usd_to_token_min(collateral_token.clone(), fee, deps.storage)?;
        _decrease_pool_amount(deps.storage, env.clone(), collateral_token.clone(), amount)?;
    } else {
        if global_short_sizes == 0 {
            global_short_average_prizes = 0;
        } else {
            global_short_average_prizes = get_next_global_short_average_price(
                index_token.clone(),
                price,
                size_delta,
                deps.storage,
            )?;
        }
    }
    increase_global_short_size(deps.storage, index_token.clone(), size_delta)?;
    POSITIONS.save(deps.storage, &key, &positions)?;

    GLOBAL_SHORT_AVERAGE_PRIZES.save(
        deps.storage,
        index_token.clone(),
        &global_short_average_prizes,
    )?;

    Ok(Response::new()
        .add_attribute("method", "increase_position")
        .add_attribute("account", account.to_string())
        .add_attribute("collateral_token", collateral_token.to_string())
        .add_attribute("index_token", index_token.to_string())
        .add_attribute("collateral_delta_usd", collateral_delta_usd.to_string())
        .add_attribute("size_delta", size_delta.to_string())
        .add_attribute("is_long", is_long.to_string())
        .add_attribute("price", price.to_string())
        .add_attribute("fee", fee.to_string()))
}

fn try_decrease_position(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    collateral_delta: u128,
    size_delta: u128,
    is_long: bool,
    reciever: Addr,
) -> Result<Response, ContractError> {
    _validate_gas_price(deps.storage, env.clone());
    _validate_router(deps.storage, info.clone(), account.clone());

    // vaultUtils.validateDecreasePosition(_account, _collateralToken, _indexToken, _collateralDelta, _sizeDelta, _isLong, _receiver); // skipped not present in cosmwasm
    update_cumulative_funding_rate(
        deps.storage,
        env.clone(),
        collateral_token.clone(),
        index_token.clone(),
    )?;

    let key = get_position_key(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long,
    )?;

    let mut position = POSITIONS.load(deps.storage, &key)?;
    _validate(position.size > 0, 31)?;
    _validate(position.size >= size_delta, 32)?;
    _validate(position.collateral >= collateral_delta, 33)?;

    let collateral = position.collateral;

    // scrop variables to avoid stack too deep errors
    {
        let rserve_delta = position.reserve_amount.mul(size_delta).div(position.size);
        position.reserve_amount = position.reserve_amount.sub(rserve_delta);
        decrease_reserved_amount(deps.storage, collateral_token.clone(), rserve_delta)?;
    }

    let (usd_out, usd_out_after_fee) = reduce_collateral(
        deps.storage,
        deps.querier,
        env.clone(),
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        collateral_delta,
        size_delta,
        is_long,
    )?;

    if position.size != size_delta {
        position.entry_funding_rate = 0; // hardcode as return vaultUtils.getEntryFundingRate(_collateralToken, _indexToken, _isLong); is not present in cosmwsm
        position.size = position.size.sub(size_delta);

        validate_position(position.size, collateral.clone());

        //      // validateLiquidation returns (state, fees)
        // function validateLiquidation(address _account, address _collateralToken, address _indexToken, bool _isLong, bool _raise) override public view returns (uint256, uint256) {
        //     return vaultUtils.validateLiquidation(_account, _collateralToken, _indexToken, _isLong, _raise);
        // }  Skipping this as not present in cosmwasm

        if is_long {
            increase_guarnteed_usd(
                deps.storage,
                collateral_token.clone(),
                collateral.sub(position.collateral),
            )?;
            decrease_guarnteed_usd(deps.storage, collateral_token.clone(), size_delta)?;
        }

        let _price = if is_long {
            get_min_price(index_token.clone())
        } else {
            get_max_price(index_token.clone())
        };

        POSITIONS.remove(deps.storage, &key);
    }

    if is_long {
        decrease_global_short_size(deps.storage, index_token.clone(), size_delta)?;
    }
    if usd_out > 0 {
        if is_long {
            let amount =
                usd_to_token_min(collateral_token.clone(), usd_out_after_fee, deps.storage)?;
            _decrease_pool_amount(deps.storage, env.clone(), collateral_token.clone(), amount)?;
        }
        let amount_out_after_fees =
            usd_to_token_min(collateral_token.clone(), usd_out_after_fee, deps.storage)?;
        _transfer_out(
            info.clone(),
            collateral_token.clone(),
            amount_out_after_fees,
            reciever.clone(),
        )?;
        return Ok(Response::new()
            .add_attribute("amount_out_after_fees", amount_out_after_fees.to_string()));
    }

    Ok(Response::new()
        .add_attribute("method", "decrease_position")
        .add_attribute("amount_out_after_fees", 0.to_string()))
}

fn try_liquidate_position(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
    _fee_reciever: Addr,
) -> Result<Response, ContractError> {
    let mut state_variables = STATE_VARIABLES.load(deps.storage)?;
    let is_liquidator = IS_LIQUIDATOR.load(deps.storage, info.sender.clone())?;
    if state_variables.in_private_liquidation_mode {
        _validate(is_liquidator, 34)?;
    }

    // set includeAmmPrice to false to prevent manipulated liquidations
    state_variables.include_amm_price = false;

    update_cumulative_funding_rate(
        deps.storage,
        env.clone(),
        collateral_token.clone(),
        index_token.clone(),
    )?;
    let key = get_position_key(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long,
    )?;
    let position = POSITIONS.load(deps.storage, &key)?;
    _validate(position.size > 0, 35)?;

    let (liquidation_state, margin_fees) = (1, 1); // hardcode as vaultUtils.validateLiquidation(_account, _collateralToken, _indexToken, _isLong, _raise); not present in cosmwasm
    _validate(liquidation_state != 0, 36)?;

    if liquidation_state == 2 {
        // max leverage exceeded but there is collateral remaining after deducting losses so decreasePosition instead
        state_variables.include_amm_price = true;
        return Ok(Response::default());
    }
    let fee_tokens = usd_to_token_min(collateral_token.clone(), margin_fees, deps.storage)?;
    let mut fee_reserves = FEE_RESERVES.load(deps.storage, collateral_token.clone())?;
    fee_reserves = fee_reserves.add(fee_tokens);
    decrease_reserved_amount(
        deps.storage,
        collateral_token.clone(),
        position.reserve_amount,
    )?;

    if is_long {
        decrease_guarnteed_usd(
            deps.storage,
            collateral_token.clone(),
            position.size.sub(position.collateral),
        )?;
        let amount = usd_to_token_min(collateral_token.clone(), margin_fees, deps.storage)?;
        _decrease_pool_amount(deps.storage, env.clone(), collateral_token.clone(), amount)?;
    }

    let _mark_price = if is_long {
        get_min_price(index_token.clone())
    } else {
        get_max_price(index_token.clone())
    };

    if is_long && margin_fees < position.collateral {
        let remaining_collateral = position.collateral.sub(margin_fees);
        let amount =
            usd_to_token_min(collateral_token.clone(), remaining_collateral, deps.storage)?;

        increase_pool_amount(
            deps.storage,
            deps.querier,
            env.clone(),
            collateral_token.clone(),
            amount,
        )?;
    }

    if !is_long {
        decrease_global_short_size(deps.storage, index_token.clone(), position.size)?;
    }

    POSITIONS.remove(deps.storage, &key);

    // pay the fee receiver using the pool, we assume that in general the liquidated amount should be sufficient to cover
    // the liquidation fees
    let amount = usd_to_token_min(
        collateral_token.clone(),
        state_variables.liquidation_fee_usd,
        deps.storage,
    )?;
    _decrease_pool_amount(deps.storage, env.clone(), collateral_token.clone(), amount)?;

    state_variables.include_amm_price = true;

    STATE_VARIABLES.save(deps.storage, &state_variables)?;

    Ok(Response::new()
        .add_attribute("method", "liquidate_position")
        .add_attribute("acount", account.to_string())
        .add_attribute("collateral_token", collateral_token.to_string())
        .add_attribute("index_token", index_token.to_string())
        .add_attribute("is_long", is_long.to_string())
        .add_attribute("size", position.size.to_string())
        .add_attribute("collateral", position.collateral.to_string())
        .add_attribute("reserve_amount", position.reserve_amount.to_string())
        .add_attribute("realised_pnl", position.realised_pnl.to_string())
        .add_attribute("mark_price", _mark_price.to_string()))
}

// Query

fn try_get_redemption_collateral(deps: DepsMut, token: Addr) -> StdResult<Binary> {
    let stable_token = STABLE_TOKENS.load(deps.storage, token.clone())?;
    let pool_amounts = POOL_AMOUNTS.load(deps.storage, token.clone())?;
    let guaranteed_usd = GUARANTEED_USD.load(deps.storage, token.clone())?;
    let reserved_amount = RSERVED_AMOUNTS.load(deps.storage, token.clone())?;
    if stable_token {
        return to_binary(&pool_amounts);
    }

    let collateral = usd_to_token_min(token.clone(), guaranteed_usd, deps.storage)?;
    let res = collateral.add(pool_amounts).sub(reserved_amount);
    to_binary(&(res))
}

fn try_get_redemption_collateral_usd(deps: DepsMut, token: Addr) -> StdResult<Binary> {
    let stable_token = STABLE_TOKENS.load(deps.storage, token.clone())?;
    let pool_amounts = POOL_AMOUNTS.load(deps.storage, token.clone())?;
    let guaranteed_usd = GUARANTEED_USD.load(deps.storage, token.clone())?;
    let reserved_amount = RSERVED_AMOUNTS.load(deps.storage, token.clone())?;
    if stable_token {
        return to_binary(&pool_amounts);
    }

    let collateral = usd_to_token_min(token.clone(), guaranteed_usd, deps.storage)?;
    let amount = collateral.add(pool_amounts).sub(reserved_amount);
    let res = token_to_usd_min(token.clone(), amount, deps.storage)?;
    to_binary(&(res))
}

fn try_get_utilisation(deps: DepsMut, token: Addr) -> StdResult<Binary> {
    let pool_amounts = POOL_AMOUNTS.load(deps.storage, token.clone())?;
    let reserved_amoints = RSERVED_AMOUNTS.load(deps.storage, token.clone())?;
    if pool_amounts == 0 {
        return to_binary(&0);
    }
    let res = reserved_amoints
        .mul(FUNDING_RATE_PRECISION)
        .div(pool_amounts);
    to_binary(&(res))
}

fn try_get_position(
    deps: DepsMut,
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
) -> StdResult<Binary> {
    let key = get_position_key(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long,
    )
    .unwrap();
    let position = POSITIONS.load(deps.storage, &key)?;
    let realised_pnl;
    if position.realised_pnl > 0 {
        realised_pnl = position.realised_pnl;
    } else {
        realised_pnl = 0 - position.realised_pnl;
    }
    let res = Position {
        size: position.size,
        collateral: position.collateral,
        average_price: position.average_price,
        entry_funding_rate: position.entry_funding_rate,
        reserve_amount: position.reserve_amount,
        realised_pnl,
        last_increased_time: position.last_increased_time,
    };
    to_binary(&(res))
}

fn try_get_position_leverage(
    deps: DepsMut,
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
) -> StdResult<Binary> {
    let key = get_position_key(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long,
    )
    .unwrap();
    let position = POSITIONS.load(deps.storage, &key)?;
    _validate(position.collateral > 0, 37).unwrap();
    let res = position
        .size
        .mul(BASIS_POINTS_DIVISOR)
        .div(position.collateral);
    to_binary(&(res))
}

fn try_global_short_delta(deps: DepsMut, token: Addr) -> StdResult<Binary> {
    let global_short_sizes = GLOBAL_SHORT_SIZES.load(deps.storage, token.clone())?;
    let global_short_average_prices =
        GLOBAL_SHORT_AVERAGE_PRIZES.load(deps.storage, token.clone())?;
    if global_short_sizes == 0 {
        let res = GlobalShortDeltaResponse {
            has_profit: false,
            delta: 0,
        };
        return to_binary(&res);
    }
    let next_price = get_max_price(token.clone());
    let mut price_delta = 0;
    if global_short_average_prices > next_price {
        price_delta = global_short_average_prices.sub(next_price);
    } else {
        price_delta = next_price.sub(global_short_average_prices);
    }
    let delta = global_short_average_prices
        .mul(price_delta)
        .div(global_short_average_prices);
    let has_profit = global_short_average_prices > next_price;
    let res = GlobalShortDeltaResponse { has_profit, delta };
    to_binary(&(res))
}

fn try_get_position_delta(
    deps: DepsMut,
    env: Env,
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
) -> StdResult<Binary> {
    let key = get_position_key(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long,
    )
    .unwrap();
    let position = POSITIONS.load(deps.storage, &key)?;

    let res = get_delta(
        index_token.clone(),
        position.size,
        position.average_price,
        is_long,
        position.last_increased_time,
        deps.storage,
        env,
    )?;
    to_binary(&(res))
}

fn try_get_target_usdg_amount(deps: DepsMut, env: Env, token: Addr) -> StdResult<Binary> {
    let supply = get_token_balance_of(
        deps.querier,
        env.contract.address.clone(),
        env.contract.address,
    )
    .unwrap();
    let token_weights = TOKEN_WEIGHTS.load(deps.storage, token)?;
    let state_variables = STATE_VARIABLES.load(deps.storage)?;
    if supply == 0 {
        return to_binary(&0);
    }
    let res = token_weights
        .mul(supply)
        .div(state_variables.total_token_weights);
    to_binary(&(res))
}

// Helper Functions

fn only_gov(storage: &mut dyn Storage, addr: Addr) {
    let gov_addr = GOV.load(storage).unwrap();
    _validate(addr == gov_addr, 53).unwrap();
}
fn _validate(_condition: bool, _error_code: u64) -> Result<Response, ContractError> {
    if !_condition {
        return Err(ContractError::CustomError {
            val: format!("Error code: {}", _error_code),
        });
    }
    Ok(Response::default())
}

fn _validate_manager(storage: &mut dyn Storage, addr: Addr) {
    let state_variables = STATE_VARIABLES.load(storage).unwrap();
    let is_manager = IS_MANAGER.load(storage, addr).unwrap();
    if state_variables.in_manager_mode {
        _validate(is_manager, 54).unwrap();
    }
}

fn all_whitelisted_tokens_length(storage: &mut dyn Storage) -> Result<usize, ContractError> {
    let state_variables = STATE_VARIABLES.load(storage)?;

    Ok(state_variables.all_whitelisted_tokens.len())
}

fn get_max_price(_token: Addr) -> u128 {
    return 1;
}

fn get_min_price(_token: Addr) -> u128 {
    return 1;
}

fn _transfer_out(
    info: MessageInfo,
    token: Addr,
    amount: u128,
    reciever: Addr,
) -> Result<Response, ContractError> {
    let tranfr_msg = cw20::Cw20ExecuteMsg::Transfer {
        recipient: reciever.to_string(),
        amount: amount.into(),
    };
    let msg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_binary(&tranfr_msg)?,
        funds: info.funds,
    });
    Ok(Response::new()
        .add_attribute("action", "transfer_out")
        .add_message(msg))
}

fn _transfer_in(
    deps: &DepsMut,
    env: Env,
    _info: MessageInfo,
    token: Addr,
) -> Result<u128, ContractError> {
    let mut token_balances = TOKEN_BALANCES.load(deps.storage, token.clone())?;
    let prev_balance = token_balances;
    let next_balance = get_token_balance_of(deps.querier, env.contract.address, token.clone())?;

    token_balances = next_balance;
    Ok(next_balance.sub(prev_balance))
}

fn _increase_usdg_amount(
    storage: &mut dyn Storage,
    token: Addr,
    amount: u128,
) -> Result<Response, ContractError> {
    let mut usdg_amount = USDG_AMOUNTS.load(storage, token.clone())?;
    let max_usdg_amount = MAX_USDG_AMOUNTS.load(storage, token.clone())?;
    usdg_amount += amount;

    if max_usdg_amount != 0 {
        _validate(usdg_amount <= max_usdg_amount, 51)?;
    }

    USDG_AMOUNTS.save(storage, token.clone(), &usdg_amount)?;

    Ok(Response::new()
        .add_attribute("action", "increase_usdg_amount")
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn _decrease_usdg_amount(
    storage: &mut dyn Storage,
    token: Addr,
    amount: u128,
) -> Result<Response, ContractError> {
    let mut usdg_amount = USDG_AMOUNTS.load(storage, token.clone())?;

    // since USDG can be minted using multiple assets
    // it is possible for the USDG debt for a single asset to be less than zero
    // the USDG debt is capped to zero for this case

    if usdg_amount <= amount {
        usdg_amount = 0;
        return Ok(Response::new()
            .add_attribute("action", "decrease_usdg_amount")
            .add_attribute("token", token.to_string())
            .add_attribute("amount", amount.to_string()));
    }
    usdg_amount -= amount;
    USDG_AMOUNTS.save(storage, token.clone(), &usdg_amount)?;

    Ok(Response::new()
        .add_attribute("action", "decrease_usdg_amount")
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string()))
}

pub fn get_token_balance_of(
    querier: QuerierWrapper,
    user_address: Addr,
    cw20_contract_addr: Addr,
) -> Result<u128, ContractError> {
    let query_msg = cw20::Cw20QueryMsg::Balance {
        address: user_address.to_string(),
    };
    let msg = querier.query(&cosmwasm_std::QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: cw20_contract_addr.to_string(),
        msg: to_binary(&query_msg)?,
    }))?;

    Ok(msg)
}

fn increase_pool_amount(
    storage: &mut dyn Storage,
    querier: QuerierWrapper,
    env: Env,
    token: Addr,
    amount: u128,
) -> Result<Response, ContractError> {
    let pool_amounts = POOL_AMOUNTS.load(storage, token.clone())?;
    let balance = get_token_balance_of(querier, env.contract.address, token.clone())?;
    _validate(pool_amounts <= balance, 49)?;
    Ok(Response::new()
        .add_attribute("action", "increase_pool_amount")
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn update_cumulative_funding_rate(
    storage: &mut dyn Storage,
    env: Env,
    _collateral_token: Addr,
    _index_token: Addr,
) -> Result<Response, ContractError> {
    let should_update = true;

    if !should_update {
        return Ok(Response::new().add_attribute("action", "update_funding_rate"));
    }
    let mut last_funding_times = LAST_FUNDING_TIMES
        .load(storage, _collateral_token.clone())
        .unwrap();
    let state_variables = STATE_VARIABLES.load(storage).unwrap();
    if last_funding_times == 0 {
        last_funding_times = env
            .block
            .time
            .seconds()
            .clone()
            .div(state_variables.funding_interval)
            .mul(state_variables.funding_interval);
        return Ok(Response::new().add_attribute("action", "update_funding_rate"));
    }
    if last_funding_times.add(state_variables.funding_interval) > env.block.time.seconds() {
        return Ok(Response::new().add_attribute("action", "update_funding_rate"));
    }

    let funding_rate =
        get_next_funding_rate(storage, env.clone(), _collateral_token.clone()).unwrap();
    let commulative_funding_rates = COMMULATIVE_FUNDING_RATES
        .load(storage, _collateral_token.clone())
        .unwrap();
    last_funding_times = env
        .block
        .time
        .seconds()
        .div(state_variables.funding_interval)
        .mul(state_variables.funding_interval);

    LAST_FUNDING_TIMES
        .save(storage, _collateral_token.clone(), &last_funding_times)
        .unwrap();

    Ok(Response::new()
        .add_attribute("action", "update_funding_rate")
        .add_attribute("collateral_token", _collateral_token.to_string())
        .add_attribute(
            "commulative_funding_rates",
            commulative_funding_rates.to_string(),
        ))
}

fn get_next_funding_rate(
    storage: &mut dyn Storage,
    env: Env,
    token: Addr,
) -> Result<u128, ContractError> {
    let mut last_funding_times = LAST_FUNDING_TIMES.load(storage, token.clone()).unwrap();
    let state_variables = STATE_VARIABLES.load(storage).unwrap();

    if last_funding_times.add(state_variables.funding_interval) > env.block.time.seconds() {
        return Ok(0);
    }
    let intervals = env
        .block
        .time
        .seconds()
        .sub(last_funding_times.div(state_variables.funding_interval));
    let pool_amounts = POOL_AMOUNTS.load(storage, token.clone())?;
    if pool_amounts == 0 {
        return Ok(0);
    }
    let is_stable = STABLE_TOKENS.load(storage, token.clone())?;
    let mut funding_rate_factor = 0;
    if is_stable {
        funding_rate_factor = state_variables.stable_funding_rate_factor;
    } else {
        funding_rate_factor = state_variables.funding_rate_factor;
    }
    let reserved_amounts = RSERVED_AMOUNTS.load(storage, token)?;

    Ok(funding_rate_factor
        .mul(reserved_amounts)
        .mul(intervals as u128)
        .div(pool_amounts))
}

fn adjust_for_decimals(
    storage: &mut dyn Storage,
    amount: u128,
    token_div: Addr,
    token_mul: Addr,
) -> Result<u128, ContractError> {
    let addresses = ADDRESSES.load(storage).unwrap();
    let mut decimals_div = 0;
    let mut decimals_mul = 0;
    if token_div == addresses.usdg {
        decimals_div = USDG_DECIMALS;
    } else {
        let token_decimals = TOKEN_DECIMALS.load(storage, token_div).unwrap();
        decimals_div = token_decimals;
    }

    if token_mul == addresses.usdg {
        decimals_mul = USDG_DECIMALS;
    } else {
        let token_decimals = TOKEN_DECIMALS.load(storage, token_mul).unwrap();
        decimals_mul = token_decimals;
    }
    Ok(amount.mul(
        10_u128
            .pow(decimals_mul as u32)
            .div(10_u128.pow(decimals_div as u32)),
    ))
}

fn _collect_swap_fees(
    storage: &mut dyn Storage,
    token: Addr,
    amount: u128,
    fee_basis_point: u128,
) -> Result<u128, ContractError> {
    let after_fee_amount = amount
        .mul(BASIS_POINTS_DIVISOR)
        .sub(fee_basis_point)
        .div(BASIS_POINTS_DIVISOR);
    let fee_amount = amount.sub(after_fee_amount);
    let mut fee_reserves = FEE_RESERVES.load(storage, token).unwrap();
    fee_reserves = fee_reserves.add(fee_amount);

    Ok(after_fee_amount)
}

fn _increase_pool_amount(
    storage: &mut dyn Storage,
    querier: QuerierWrapper,
    env: Env,
    token: Addr,
    amount: u128,
) -> Result<Response, ContractError> {
    let mut pool_amounts = POOL_AMOUNTS.load(storage, token.clone())?;
    pool_amounts.add(amount);
    let balance = get_token_balance_of(querier, env.contract.address, token.clone())?;
    _validate(pool_amounts <= balance, 49)?;

    POOL_AMOUNTS.save(storage, token.clone(), &pool_amounts)?;

    Ok(Response::new()
        .add_attribute("action", "increase_pool_amount")
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn get_redemption_amount(
    storage: &mut dyn Storage,
    _env: Env,
    token: Addr,
    usdg_amount: u128,
) -> Result<u128, ContractError> {
    let price = get_max_price(token.clone());
    let redemption_amount = usdg_amount.mul(PRICE_PRECISION).div(price);
    let addresses = ADDRESSES.load(storage).unwrap();

    Ok(adjust_for_decimals(
        storage,
        redemption_amount,
        addresses.usdg,
        token,
    )?)
}

fn _decrease_pool_amount(
    storage: &mut dyn Storage,
    env: Env,
    token: Addr,
    amount: u128,
) -> Result<Response, ContractError> {
    let mut pool_amounts = POOL_AMOUNTS.load(storage, token.clone())?;
    let reserved_amounts = RSERVED_AMOUNTS.load(storage, token.clone())?;
    pool_amounts.sub(amount);
    _validate(reserved_amounts <= pool_amounts, 50)?;

    POOL_AMOUNTS.save(storage, token.clone(), &pool_amounts)?;

    Ok(Response::new()
        .add_attribute("action", "decrease_pool_amount")
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn _update_token_balance(
    storage: &mut dyn Storage,
    querier: QuerierWrapper,
    env: Env,
    token: Addr,
) -> Result<Response, ContractError> {
    let next_balance = get_token_balance_of(querier, token.clone(), env.contract.address)?;
    let mut token_balances = TOKEN_BALANCES.load(storage, token.clone())?;
    token_balances = next_balance;
    TOKEN_BALANCES.save(storage, token, &token_balances)?;
    Ok(Response::default())
}

fn _validate_buffer_amount(
    storage: &mut dyn Storage,
    token: Addr,
) -> Result<Response, ContractError> {
    let pool_amounts = POOL_AMOUNTS.load(storage, token.clone()).unwrap();
    let buffer_amounts = BUFFER_AMOUNTS.load(storage, token.clone()).unwrap();

    if pool_amounts < buffer_amounts {
        return Err(ContractError::CustomError {
            val: "Vault: poolAmount < buffer".to_string(),
        });
    }
    Ok(Response::default())
}

fn _validate_gas_price(storage: &mut dyn Storage, env: Env) {
    let state_variables = STATE_VARIABLES.load(storage).unwrap();
    if state_variables.max_gas_price == 0 {
        return;
    }
    let tx_gas_price = 1;
    _validate(tx_gas_price <= state_variables.max_gas_price, 55).unwrap();
}

fn _validate_router(storage: &mut dyn Storage, info: MessageInfo, account: Addr) {
    let addresses = ADDRESSES.load(storage).unwrap();
    let approved_routers1 = APPROVED_ROUTERS1.load(storage, info.sender).unwrap();
    let mut approved_routers2 = APPROVED_ROUTERS2.load(storage, account).unwrap();
    approved_routers2 = approved_routers1;
    _validate(approved_routers2, 41).unwrap();
}

fn validate_tokens(
    storage: &mut dyn Storage,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
) {
    let whitelisted_tokens = WHITELISTED_TOKENS
        .load(storage, collateral_token.clone())
        .unwrap();
    let stable_tokens_collateral = STABLE_TOKENS
        .load(storage, collateral_token.clone())
        .unwrap();
    let stable_tokens_index = STABLE_TOKENS.load(storage, index_token.clone()).unwrap();
    let shortable_tokens = SHORTABLE_TOKENS.load(storage, index_token.clone()).unwrap();
    if is_long {
        _validate(collateral_token == index_token, 42).unwrap();
        _validate(whitelisted_tokens, 43).unwrap();
        _validate(!stable_tokens_collateral, 44).unwrap();
        return;
    }
    _validate(whitelisted_tokens, 45).unwrap();
    _validate(stable_tokens_collateral, 46).unwrap();
    _validate(!stable_tokens_index, 47).unwrap();
    _validate(shortable_tokens, 48).unwrap();
}

fn get_position_key(
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
) -> Result<Vec<u8>, ContractError> {
    let key = Key::new(account, collateral_token, index_token, is_long).as_bytes()?;
    Ok(key)
}

fn get_next_average_price(
    index_token: Addr,
    size: u128,
    average_price: u128,
    is_long: bool,
    next_price: u128,
    size_delta: u128,
    last_increased_time: u128,
    env: Env,
    storage: &mut dyn Storage,
) -> StdResult<u128> {
    let (has_profit, delta) = get_delta(
        index_token,
        size,
        average_price,
        is_long,
        last_increased_time,
        storage,
        env,
    )?;
    let next_size = size + size_delta;
    let divisor = if is_long {
        if has_profit {
            next_size + delta
        } else {
            next_size - delta
        }
    } else {
        if has_profit {
            next_size - delta
        } else {
            next_size + delta
        }
    };

    let result = next_price * next_size / divisor;
    Ok(result)
}

fn get_delta(
    index_token: Addr,
    size: u128,
    average_price: u128,
    is_long: bool,
    last_increased_time: u128,
    storage: &mut dyn Storage,
    env: Env,
) -> StdResult<(bool, u128)> {
    let state_variables = STATE_VARIABLES.load(storage).unwrap();
    let min_profit_basis_points = MIN_PROFIT_BASIS_POINTS
        .load(storage, index_token.clone())
        .unwrap();

    // Get the appropriate price based on the is_long flag
    let price = if is_long {
        0
    } else {
        get_max_price(index_token)
    };

    let price_delta = if average_price > price {
        average_price - price
    } else {
        price - average_price
    };

    let delta = size * price_delta / average_price;

    let has_profit = if is_long {
        price > average_price
    } else {
        average_price > price
    };

    // Additional logic involving minProfitTime and minProfitBasisPoints
    let min_bps = if env.block.time.seconds() as u128
        > last_increased_time + state_variables.min_profit_time
    {
        0
    } else {
        min_profit_basis_points
    };

    let adjusted_delta = if has_profit && delta * BASIS_POINTS_DIVISOR <= size * min_bps {
        0
    } else {
        delta
    };

    Ok((has_profit, adjusted_delta))
}

fn collect_margin_fees(
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    is_long: bool,
    size_delta: u128,
    size: u128,
    entry_funding_rate: u128,
    storage: &mut dyn Storage,
) -> StdResult<u128> {
    let fee_reserves = FEE_RESERVES
        .load(storage, collateral_token.clone())
        .unwrap();
    let mut fee_usd = 0; // Hardcoded

    let funding_fee = 0; // Hardcoded
    fee_usd += funding_fee;

    let fee_tokens = usd_to_token_min(collateral_token.clone(), fee_usd, storage)?;
    fee_reserves.add(fee_tokens);
    FEE_RESERVES
        .save(storage, collateral_token, &fee_reserves)
        .unwrap();

    Ok(fee_usd)
}

fn usd_to_token_min(token: Addr, usd_amount: u128, storage: &mut dyn Storage) -> StdResult<u128> {
    if usd_amount == 0 {
        return Ok(0);
    }
    return usd_to_token(token.clone(), usd_amount, get_max_price(token), storage);
}

fn usd_to_token_max(token: Addr, usd_amount: u128, storage: &mut dyn Storage) -> StdResult<u128> {
    if usd_amount == 0 {
        return Ok(0);
    }
    return usd_to_token(token.clone(), usd_amount, get_min_price(token), storage);
}

fn usd_to_token(
    token: Addr,
    usd_amount: u128,
    price: u128,
    storage: &mut dyn Storage,
) -> StdResult<u128> {
    let token_decimals = TOKEN_DECIMALS.load(storage, token.clone()).unwrap();
    if usd_amount == 0 {
        return Ok(0);
    }
    let decimals = token_decimals;
    Ok(usd_amount.mul(10_u128.pow(decimals as u32).div(price)))
}

fn token_to_usd_min(token: Addr, token_amount: u128, storage: &mut dyn Storage) -> StdResult<u128> {
    let token_decimals = TOKEN_DECIMALS.load(storage, token.clone()).unwrap();
    if token_amount == 0 {
        return Ok(0);
    }
    let price = 0; // hardcoded
    let decimals = token_decimals;
    return Ok(token_amount.mul(price).div(10_u128.pow(decimals as u32)));
}

fn validate_position(size: u128, collateral: u128) {
    if size == 0 {
        _validate(collateral == 0, 39).unwrap();
        return;
    }
    _validate(size >= collateral, 40).unwrap();
}

fn increase_reserved_amount(
    storage: &mut dyn Storage,
    token: Addr,
    amount: u128,
) -> Result<Response, ContractError> {
    let mut reserved_amounts = RSERVED_AMOUNTS.load(storage, token.clone()).unwrap();
    let mut pool_amounts = POOL_AMOUNTS.load(storage, token.clone()).unwrap();
    reserved_amounts.add(amount);
    _validate(reserved_amounts <= pool_amounts, 52)?;
    RSERVED_AMOUNTS
        .save(storage, token.clone(), &reserved_amounts)
        .unwrap();

    Ok(Response::new()
        .add_attribute("acton", "increase_reserve_amount")
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string()))
}

fn decrease_reserved_amount(
    storage: &mut dyn Storage,
    token: Addr,
    amount: u128,
) -> Result<Response, ContractError> {
    let mut reserved_amounts = RSERVED_AMOUNTS.load(storage, token.clone()).unwrap();
    reserved_amounts.sub(amount);
    RSERVED_AMOUNTS
        .save(storage, token.clone(), &reserved_amounts)
        .unwrap();

    Ok(Response::new()
        .add_attribute("acton", "decrease_reserve_amount")
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string()))
}
fn increase_guarnteed_usd(
    storage: &mut dyn Storage,
    token: Addr,
    usdg_amount: u128,
) -> Result<Response, ContractError> {
    let mut guarnteed_usd = GUARANTEED_USD.load(storage, token.clone()).unwrap();
    guarnteed_usd.add(usdg_amount);
    GUARANTEED_USD
        .save(storage, token.clone(), &guarnteed_usd)
        .unwrap();
    Ok(Response::new()
        .add_attribute("acton", "increase_guarnteed_usd")
        .add_attribute("token", token.to_string())
        .add_attribute("usdg_aount", usdg_amount.to_string()))
}

fn decrease_guarnteed_usd(
    storage: &mut dyn Storage,
    token: Addr,
    usdg_amount: u128,
) -> Result<Response, ContractError> {
    let mut guarnteed_usd = GUARANTEED_USD.load(storage, token.clone()).unwrap();
    guarnteed_usd.sub(usdg_amount);
    GUARANTEED_USD
        .save(storage, token.clone(), &guarnteed_usd)
        .unwrap();
    Ok(Response::new()
        .add_attribute("acton", "decrese_guarnteed_usd")
        .add_attribute("token", token.to_string())
        .add_attribute("usdg_aount", usdg_amount.to_string()))
}

fn get_next_global_short_average_price(
    index_token: Addr,
    next_price: u128,
    size_delta: u128,
    storage: &mut dyn Storage,
) -> StdResult<u128> {
    let global_short_sizes = GLOBAL_SHORT_SIZES.load(storage, index_token.clone())?;
    let mut global_short_average_prizes =
        GLOBAL_SHORT_AVERAGE_PRIZES.load(storage, index_token.clone())?;
    let size = global_short_sizes;
    let average_price = global_short_average_prizes;
    let price_delta = if average_price > next_price {
        average_price - next_price
    } else {
        next_price - average_price
    };

    let delta = size * price_delta / average_price;
    let has_profit = average_price > next_price;

    let next_size = size + size_delta;
    let divisor = if has_profit {
        next_size - delta
    } else {
        next_size + delta
    };

    let result = next_price * next_size / divisor;
    Ok(result)
}

fn increase_global_short_size(
    storage: &mut dyn Storage,
    index_token: Addr,
    amount: u128,
) -> StdResult<()> {
    let mut global_short_sizes = GLOBAL_SHORT_SIZES.load(storage, index_token.clone())?;
    let mut max_global_short_sizes = MAX_GLOBAL_SHORT_SIZES.load(storage, index_token.clone())?;

    global_short_sizes.add(amount);
    let max_size = max_global_short_sizes;
    GLOBAL_SHORT_SIZES.save(storage, index_token.clone(), &global_short_sizes)?;
    if max_size != 0 {
        if global_short_sizes <= max_size {
            return Err(StdError::GenericErr {
                msg: "Vault: max shorts exceeded".to_string(),
            });
        }
    }

    Ok(())
}

fn decrease_global_short_size(
    storage: &mut dyn Storage,
    index_token: Addr,
    amount: u128,
) -> StdResult<()> {
    let mut global_short_sizes = GLOBAL_SHORT_SIZES.load(storage, index_token.clone())?;

    let size = global_short_sizes;
    if amount > size {
        global_short_sizes = 0;
        return Ok(());
    }
    global_short_sizes.sub(amount);
    GLOBAL_SHORT_SIZES.save(storage, index_token, &global_short_sizes)?;
    Ok(())
}

fn reduce_collateral(
    storage: &mut dyn Storage,
    querier: QuerierWrapper,
    env: Env,
    account: Addr,
    collateral_token: Addr,
    index_token: Addr,
    collateral_delta: u128,
    size_delta: u128,
    is_long: bool,
) -> Result<(u128, u128), ContractError> {
    let key = get_position_key(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long,
    )?;
    let mut position = POSITIONS.load(storage, &key).unwrap();

    let fee = collect_margin_fees(
        account.clone(),
        collateral_token.clone(),
        index_token.clone(),
        is_long,
        size_delta,
        position.size,
        position.entry_funding_rate,
        storage,
    )?;

    let has_profit;
    let adjusted_delta;

    {
        let (has_profit_value, delta) = get_delta(
            index_token.clone(),
            position.size,
            position.average_price,
            is_long,
            position.last_increased_time,
            storage,
            env.clone(),
        )?;
        has_profit = has_profit_value;
        adjusted_delta = size_delta * delta / position.size;
    }

    let mut usd_out: u128 = 0;

    if has_profit && adjusted_delta > 0 {
        usd_out = adjusted_delta;
        position.realised_pnl += adjusted_delta;

        if !is_long {
            let token_amount = usd_to_token_min(collateral_token.clone(), adjusted_delta, storage)?;
            _decrease_pool_amount(storage, env.clone(), collateral_token.clone(), token_amount)?;
        }
    }

    if !has_profit && adjusted_delta > 0 {
        position.collateral -= adjusted_delta;

        if !is_long {
            let token_amount = usd_to_token_min(collateral_token.clone(), adjusted_delta, storage)?;
            increase_pool_amount(
                storage,
                querier,
                env.clone(),
                collateral_token.clone(),
                token_amount,
            )?;
        }

        position.realised_pnl -= adjusted_delta;
    }

    if collateral_delta > 0 {
        usd_out += collateral_delta;
        position.collateral -= collateral_delta;
    }

    if position.size == size_delta {
        usd_out += position.collateral;
        position.collateral = 0;
    }

    let usd_out_after_fee: u128 = if usd_out > fee {
        usd_out - fee
    } else {
        position.collateral -= fee;
        if is_long {
            let fee_tokens = usd_to_token_min(collateral_token.clone(), fee, storage)?;
            _decrease_pool_amount(storage, env.clone(), collateral_token, fee_tokens)?;
        }
        usd_out
    };

    Ok((usd_out, usd_out_after_fee))
}
