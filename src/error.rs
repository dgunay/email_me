#[derive(Debug)]
pub enum Error {
    Generic(String)
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Generic(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Generic(e.to_string())
    }
}

impl From<rusoto_core::region::ParseRegionError> for Error {
    fn from(e: rusoto_core::region::ParseRegionError) -> Self {
        Error::Generic(e.to_string())
    }
}

impl From<rusoto_core::credential::CredentialsError> for Error {
    fn from(e: rusoto_core::credential::CredentialsError) -> Self {
        Error::Generic(e.to_string())
    }
}

impl<T: std::error::Error + 'static> From<rusoto_core::RusotoError<T>> for Error {
    fn from(e: rusoto_core::RusotoError<T>) -> Self {
        Error::Generic(format!("{}", e))
    }
}