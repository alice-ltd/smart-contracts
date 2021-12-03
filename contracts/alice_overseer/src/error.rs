use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contract admin is not the overseer")]
    ContractAdminNotOverseer(),

    #[error("Contract address not registered with overseer")]
    UnregisteredContract(),

    #[error("Timelock already started")]
    ExistingTimelock(),

    #[error("Timelock not found")]
    TimelockNotFound(),

    #[error("Timelock not expired yet")]
    TimelockNotExpired(),
}
