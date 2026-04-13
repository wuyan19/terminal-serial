#[derive(Debug, thiserror::Error)]
pub enum SerialError {
    #[error("Failed to open {port}: {source}")]
    PortOpen {
        port: String,
        source: serialport::Error,
    },

    #[error("Write error: {0}")]
    Write(#[source] std::io::Error),

    #[error("Read error: {0}")]
    Read(#[source] std::io::Error),

    #[error("Buffer error: {0}")]
    Buffer(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}
