use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid syntax")]
    InvalidSyntax,
    #[error("Invalid calculation")]
    InvalidCalculation,
    #[error("Invalid argument")]
    IntegerOverflow,
    #[error("Invalid argument")]
    ExecutionLimitExceeded,
    #[error("Invalid key")]
    InvalidKey,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Error: {0}")]
    Custom(String),
}
