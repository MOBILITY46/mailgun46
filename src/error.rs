use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendError {
    Http(String),
    Non200Reply(reqwest::StatusCode),
    MissingField(&'static str),
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "SendingError http `{}`", msg),
            Self::MissingField(field) => write!(f, "Missing field `{}`", field),
            Self::Non200Reply(status) => {
                write!(f, "Got non 200 reply from mailgun: `{}`", status)
            }
        }
    }
}

impl std::error::Error for SendError {}

impl From<reqwest::Error> for SendError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupError {
    EnvVarMissing(&'static str),
    InvalidVar(&'static str, String),
    Build(String),
}

impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EnvVarMissing(var) => write!(f, "Missing env variable `{}`", var),
            Self::InvalidVar(var, msg) => write!(f, "Invalid value for `{}`: {}", var, msg),
            Self::Build(msg) => write!(f, "Creating Http Client: {}", msg),
        }
    }
}

impl std::error::Error for SetupError {}
