use crate::channel::patterns::{
    default_auth_passphrase_pattern,
    default_auth_password_pattern,
    default_auth_username_pattern,
    default_comms_prompt_pattern,
};

use super::constants::{
    DEFAULT_PROMPT_SEARCH_DEPTH,
    DEFAULT_READ_DELAY,
    DEFAULT_RETURN_CHAR,
    DEFAULT_TIMEOUT_OPS,
};
use core::time::Duration;
use regex::bytes::Regex;

/// A struct to hold args/settings for a `Channel` object.
#[allow(clippy::module_name_repetitions)]
pub struct Args {
    /// Indicates if we should bypass in channel authentication or not.
    pub auth_bypass: bool,
    /// Depth we should search back for the promp.
    pub prompt_search_depth: u16,
    ///  Regex pattern used to find the prompt.
    pub prompt_pattern: Regex,
    /// Return character used to... send returns.
    pub return_char: String,
    /// Pattern used to find the username prompt during in channel authentication.
    pub username_pattern: Regex,
    /// Pattern used to find the password prompt during in channel authentication.
    pub password_pattern: Regex,
    /// Pattern used to find the ssh key passphrase prompt during in channel authentication.
    pub passphrase_pattern: Regex,
    /// Delay between reads of the underlying transport.
    pub read_delay: Duration,
    /// Duration for `timeout_ops` -- the timeout for channel send operations.
    pub timeout_ops: Duration,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            auth_bypass: false,
            prompt_search_depth: DEFAULT_PROMPT_SEARCH_DEPTH,
            prompt_pattern: default_comms_prompt_pattern(),
            return_char: DEFAULT_RETURN_CHAR.to_owned(),
            username_pattern: default_auth_username_pattern(),
            password_pattern: default_auth_password_pattern(),
            passphrase_pattern: default_auth_passphrase_pattern(),
            read_delay: DEFAULT_READ_DELAY,
            timeout_ops: DEFAULT_TIMEOUT_OPS,
        }
    }
}
