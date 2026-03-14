use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum BjtError {
    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("加密错误: {0}")]
    CryptoError(String),

    #[error("文件操作错误: {0}")]
    FileError(String),

    #[error("密码错误: {0}")]
    PasswordError(String),

    #[error("密钥错误: {0}")]
    KeyError(String),

    #[error("备份错误: {0}")]
    BackupError(String),

    #[error("输入验证错误: {0}")]
    ValidationError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML解析错误: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("TOML序列化错误: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("Base64解码错误: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("十六进制解码错误: {0}")]
    HexError(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, BjtError>;