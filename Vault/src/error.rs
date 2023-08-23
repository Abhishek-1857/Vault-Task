use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("You are not the owner")]
    Unauthorized {},

    #[error("Trying to withdraw zero token")]
    InvalidAmount {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Not Enough token Staked: {staked:?} , To_Withdraw: {requested:?}")]
    NotEnoughShares { staked: Uint128, requested: Uint128 },

    #[error(" The denom you provided is incorrect")]
    IncorrectDenomination {},

    #[error(" Locking period is not over")]
    TokenLocked {},

    #[error("Error while serializing denom: `{denom}` & address: `{address}`!")]
    SerializationFailed { denom: String, address: String },

    #[error("Failed to deserialize into struct!")]
    DeserializationFailed {},
}
