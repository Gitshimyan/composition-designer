use std::error::Error;
use std::fmt;

use crate::error_code::ErrorCode;

/// 领域层不变量错误：稳定 `code` + 面向人的说明。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    code: ErrorCode,
    message: String,
}

impl DomainError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for DomainError {}

pub type DomainResult<T> = Result<T, DomainError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_error_exposes_code_and_message() {
        let error = DomainError::new(ErrorCode::SchemaInconsistency, "required + default");
        assert_eq!(error.code(), ErrorCode::SchemaInconsistency);
        assert_eq!(error.message(), "required + default");
        assert_eq!(
            error.to_string(),
            "schema-inconsistency: required + default"
        );
    }
}
