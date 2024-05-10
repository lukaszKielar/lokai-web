use derive_more::From;
use leptos::ServerFnError;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    DatabaseError(sqlx::Error),
    #[from]
    ReqwestError(reqwest::Error),
}

impl Into<ServerFnError> for Error {
    fn into(self) -> ServerFnError {
        let msg = match self {
            Error::DatabaseError(_) => "db error",
            Error::ReqwestError(_) => "reqwest error",
        }
        .to_string();
        ServerFnError::ServerError(msg)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
