use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum Error<KeyError, ValueError> {
    Fjall(fjall::Error),
    Key(KeyError),
    Value(ValueError),
}

impl<KeyError: StdError, ValueError: StdError> Error<KeyError, ValueError> {
    pub fn unwrap_fjall(self) -> fjall::Error {
        match self {
            Error::Fjall(error) => error,
            Error::Key(_) => panic!("Unwrapped a non fjall error"),
            Error::Value(_) => panic!("Unwrapped a non fjall error"),
        }
    }
}

impl<KeyError: StdError, ValueError: StdError> fmt::Display for Error<KeyError, ValueError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Fjall(error) => write!(f, "{error}"),
            Error::Key(error) => write!(f, "{error}"),
            Error::Value(error) => write!(f, "{error}"),
        }
    }
}
impl<KeyError: StdError, ValueError: StdError> StdError for Error<KeyError, ValueError> {}
