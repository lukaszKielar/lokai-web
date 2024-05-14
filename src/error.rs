use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    DatabaseError(sqlx::Error),
    #[from]
    ReqwestError(reqwest::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
