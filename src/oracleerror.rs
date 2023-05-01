use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum OracleError {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    DataNotFound,
}

impl Display for OracleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OracleError::Reqwest(ref err) => err.fmt(f),
            OracleError::SerdeJson(ref err) => err.fmt(f),
            OracleError::DataNotFound => panic!("Data not found"),
        }
    }
}

impl Error for OracleError {}

impl From<reqwest::Error> for OracleError {
    fn from(err: reqwest::Error) -> OracleError {
        OracleError::Reqwest(err)
    }
}

impl From<serde_json::Error> for OracleError {
    fn from(err: serde_json::Error) -> OracleError {
        OracleError::SerdeJson(err)
    }
}
