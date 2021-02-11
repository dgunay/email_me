

// #[derive(Debug)]
// pub enum Error {
//     Generic(String),
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

// impl std::error::Error for Error {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         None
//     }

//     // fn type_id(&self, _: private::Internal) -> std::any::TypeId
//     // where
//     //     Self: 'static,
//     // {
//     //     std::any::TypeId::of::<Self>()
//     // }

//     // fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
//     //     None
//     // }

//     fn description(&self) -> &str {
//         "description() is deprecated; use Display"
//     }

//     // fn cause(&self) -> Option<&dyn std::error::Error> {
//     //     std::error.source()
//     // }
// }

// impl From<serde_json::Error> for Error {
//     fn from(e: serde_json::Error) -> Self {
//         Error::Generic(e.to_string())
//     }
// }

// impl From<std::io::Error> for Error {
//     fn from(e: std::io::Error) -> Self {
//         Error::Generic(e.to_string())
//     }
// }

// impl From<rusoto_core::region::ParseRegionError> for Error {
//     fn from(e: rusoto_core::region::ParseRegionError) -> Self {
//         Error::Generic(e.to_string())
//     }
// }

// impl From<rusoto_core::credential::CredentialsError> for Error {
//     fn from(e: rusoto_core::credential::CredentialsError) -> Self {
//         Error::Generic(e.to_string())
//     }
// }

// impl<T: std::error::Error + 'static> From<rusoto_core::RusotoError<T>> for Error {
//     fn from(e: rusoto_core::RusotoError<T>) -> Self {
//         Error::Generic(format!("{}", e))
//     }
// }
