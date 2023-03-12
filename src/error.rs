use std::error::Error;

pub type Result<'a, T> = std::result::Result<T, AppError<'a>>;

pub struct AppError<'a> {
    pub msg: String,
    pub e: Option<Box<dyn Error + Send + 'a>>,
}

impl<'a> AppError<'a> {
    pub fn new<E: Error + Send + 'a>(msg: impl Into<String>, e: E) -> AppError<'a> {
        AppError {
            msg: msg.into(),
            e: Some(Box::new(e)),
        }
    }

    pub fn msg(msg: impl Into<String>) -> AppError<'a> {
        AppError {
            msg: msg.into(),
            e: None,
        }
    }

    pub fn error<E: Error + Send + 'a>(e: E) -> AppError<'a> {
        AppError {
            msg: e.to_string(),
            e: Some(Box::new(e)),
        }
    }
}
