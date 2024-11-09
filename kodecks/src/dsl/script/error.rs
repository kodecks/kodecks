use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid syntax")]
    InvalidSyntax,
    #[error("Invalid calculation")]
    InvalidCalculation,
    #[error("Integer overflow")]
    IntegerOverflow,
    #[error("Execution limit exceeded")]
    ExecutionLimitExceeded,
    #[error("Undefined variable")]
    UndefinedVariable,
    #[error("Undefined filter")]
    UndefinedFilter,
    #[error("Invalid key")]
    InvalidKey,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid conversion")]
    InvalidConversion,
    #[error("Invalid iteration")]
    InvalidIteration,
    #[error("Invalid argument count")]
    InvalidArgumentCount,
    #[error("Error: {0}")]
    Custom(String),
}
