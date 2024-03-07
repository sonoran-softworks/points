use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Proxy address is not valid")]
    InvalidProxyAddress,

    #[error("UnauthorizedReceive")]
    UnauthorizedReceive {},

    #[error("JobIdAlreadyPresent")]
    JobIdAlreadyPresent {},

    #[error("InvalidRandomness")]
    InvalidRandomness {},

    #[error("NoRandomnessAvailable")]
    NoRandomnessAvailable {},
}
