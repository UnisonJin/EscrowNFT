use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
   
    #[error("The price must be greater than zero")]
    NotEnoughFunds{},

    #[error("This escrow is expired")]
    EscrowExpired{},

    #[error("Escrow is not expired")]
    EscrowNotExpired{  },

    #[error("There is no such escrow")]
    NoEscrow{},

    #[error("Expected ujuno got {denom} ")]
    NotExpectedDenom{
        denom:String
    },

    #[error("You should send only one coin.")]
    OnlyOneCoinAvailable{},

    #[error("This transaction does not need any payment.")]
    NonPayable{},

}
