use std::fmt;

/// Error occuring when building a Mailer instance.
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

/// Errors occuring when building an Email from EmailBuilder::build
/// Missing fields etc..
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// A required field missing.
    MissingField(&'static str),
}
impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "Missing field `{}`", field),
        }
    }
}

impl std::error::Error for BuildError {}

/// Errors occuring when building an Email from EmailBuilder::build
/// Network errors and unexpected replies from Mailgun
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendError {
    /// Http protocol error
    Http(String),

    /// Unexpected reply from Mailgun.
    Non200Reply(reqwest::StatusCode),
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "SendingError http `{}`", msg),
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
