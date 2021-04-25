use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

// 错误
#[derive(Debug, PartialEq)]
pub enum Error {
    // 单个token为空，包括整个key为空的情况，保存对应的key
    EmptyToken(String),
    // 多层wildcard后跟随有token，保存对应key
    TokenAfterMwc(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyToken(s) => write!(f, "{} has empty token", s),
            Self::TokenAfterMwc(s) => write!(f, "{} has token after multi-layer wildcard token", s)
        }
    }
}

impl std::error::Error for Error {} 