use std::time::Duration;

///  The default depth to search backward when looking for a device "prompt".
pub const DEFAULT_PROMPT_SEARCH_DEPTH: u16 = 1024;

/// The default return character, typically this is fine, sometimes users may need to set this
/// on a given driver instance (typically to \r\n if the default is not working).
pub const DEFAULT_RETURN_CHAR: &str = "\n";

/// The default delay between reads from the underlying transport object.
pub const DEFAULT_READ_DELAY: Duration = Duration::from_micros(250);

/// The ANSI escape byte.
pub const ANSI_ESCAPE_BYTE: u8 = 0x1b;

/// A newline character as a byte.
pub const NEW_LINE_BYTE: u8 = 0x0a;

/// Constant to indicate what the "max seen" username prompts is.
pub const USER_SEEN_MAX: u8 = 2;

/// Constant to indicate what the "max seen" password prompts is.
pub const PASSWORD_SEEN_MAX: u8 = 2;

/// Constant to indicate what the "max seen" (ssh key) passphrase prompts is.
pub const PASSPHRASE_SEEN_MAX: u8 = 2;

/// Default "strip prompt" value (yes, strip the prompt by default).
pub const DEFAULT_STRIP_PROMPT: bool = true;

/// Default `timeout_ops` value.
pub const DEFAULT_TIMEOUT_OPS: Duration = Duration::from_secs(30);
