use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConverterError {
    #[error("conversion {from} -> {to} not supported: {reason}. Hint: {hint}")]
    NotSupported {
        from: String,
        to: String,
        reason: String,
        hint: String,
    },
    #[error("external tool '{tool}' not found. Install: {install_hint}")]
    ToolMissing { tool: String, install_hint: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("external tool failed ({tool}): {stderr}")]
    ToolFailed { tool: String, stderr: String },
    #[error("cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, ConverterError>;
