use cosmwasm_std::StdError;
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

    #[error("Error while serializing denom: `{denom}` & address: `{address}`!")]
    SerializationFailed { denom: String, address: String },

    #[error("Failed to deserialize into struct!")]
    DeserializationFailed {},
}
