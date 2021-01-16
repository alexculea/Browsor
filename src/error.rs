#[allow(dead_code)]
pub type BSResult<T> = std::result::Result<T, BSError>;

#[derive(Debug)]
pub struct BSError {
    msg: String,
}

impl BSError {
    pub fn new(msg: &str) -> BSError {
        BSError {
            msg: String::from(msg),
        }
    }
}

impl std::fmt::Display for BSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg.as_str())
    }
}

impl From<winrt::Error> for BSError {
    fn from(err: winrt::Error) -> Self {
        BSError::new(format!("[WinRT error] code: {} {}", err.code().0, err.message(),).as_str())
    }
}

impl From<simple_error::SimpleError> for BSError {
    fn from(err: simple_error::SimpleError) -> Self {
        BSError::new(err.as_str())
    }
}

impl From<&str> for BSError {
    fn from(str: &str) -> Self {
        BSError::new(str)
    }
}
