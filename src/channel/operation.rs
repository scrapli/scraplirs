use super::constants::DEFAULT_STRIP_PROMPT;
use core::time::Duration;
use regex::bytes::Regex;

/// Holds options to use when handling input to the `Channel`.
#[derive(Clone)]
pub struct Options {
    /// Indicates if the prompt should be stripped out of the return output or not. This defaults to
    /// *true* (yes, *do* strip the prompt out).
    pub strip_prompt: bool,
    /// Eagerly send the input -- as in send it and do not wait to return to the expected prompt,
    /// this should only be used by the netconf driver.
    pub eager: bool,
    /// Timeout to use for the operation, overrides (if set) the default channel ops timeout.
    pub timeout: Option<Duration>,
    /// Vec of regex indicating patterns that signify an operation is complete.
    pub complete_patterns: Vec<Regex>,
    /// Vec of regex indicating patterns that are valid "completion" patterns *during* an operation.
    pub interim_prompt_patterns: Vec<Regex>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            strip_prompt: DEFAULT_STRIP_PROMPT,
            eager: false,
            timeout: None,
            complete_patterns: vec![],
            interim_prompt_patterns: vec![],
        }
    }
}
