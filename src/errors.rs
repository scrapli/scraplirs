use core::fmt::{
    Display,
    Formatter,
    Result,
};
use std::error::Error;

///  `ScrapliError` is a base error for all scraplirs errors.
#[derive(Debug)]
pub struct ScrapliError {
    /// A string holding details about the error.
    pub details: String,
}

impl Display for ScrapliError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ScrapliError {
    fn description(&self) -> &str {
        &self.details
    }
}
