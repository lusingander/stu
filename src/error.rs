use std::error::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub struct AppError {
    pub msg: String,
    pub cause: Option<Box<dyn Error + Send + 'static>>,
}

impl AppError {
    pub fn new<E: Error + Send + 'static>(msg: impl Into<String>, e: E) -> AppError {
        AppError {
            msg: msg.into(),
            cause: Some(Box::new(e)),
        }
    }

    pub fn msg(msg: impl Into<String>) -> AppError {
        AppError {
            msg: msg.into(),
            cause: None,
        }
    }

    pub fn error<E: Error + Send + 'static>(e: E) -> AppError {
        AppError {
            msg: e.to_string(),
            cause: Some(Box::new(e)),
        }
    }
}
