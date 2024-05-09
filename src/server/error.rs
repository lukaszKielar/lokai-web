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
        todo!()
    }
}

pub type Result<T> = core::result::Result<T, Error>;
