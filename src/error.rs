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

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}
