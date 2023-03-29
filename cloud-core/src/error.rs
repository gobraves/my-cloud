use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    //#[error("I/O error")]
    //Io(#[from] std::io::Error),
    //#[error("JSON error")]
    //Json(#[from] serde_json::Error),
    //#[error("Invalid input")]
    //InvalidInput,
    #[error("hash not consistent")]
    HashCheckError(String),
}
