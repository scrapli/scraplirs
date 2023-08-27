use once_cell::sync::OnceCell;
use regex::bytes::Regex;

/// # Panics
///
///  Returns (once), the complied default prompt pattern. This should realisitcally never panic.
#[allow(clippy::expect_used)]
pub fn default_comms_prompt_pattern() -> Regex {
    static RE: OnceCell<Regex> = OnceCell::new();

    RE.get_or_init(|| {
        Regex::new(r"(?im)^[a-z\d.\-@()/:]{1,48}[#>$]\s*$")
            .expect("failed compiling pattern, this is a bug")
    })
    .clone()
}

/// # Panics
///
///  Returns (once), the complied default username (logon) pattern. This should realisitcally
///  never panic.
#[allow(clippy::expect_used)]
pub fn default_auth_username_pattern() -> Regex {
    static RE: OnceCell<Regex> = OnceCell::new();

    RE.get_or_init(|| {
        Regex::new(r"(?im)^(.*username:)|(.*login:)\s?$")
            .expect("failed compiling pattern, this is a bug")
    })
    .clone()
}

/// # Panics
///
///  Returns (once), the complied default password (logon) pattern. This should realisitcally
///  never panic.
#[allow(clippy::expect_used)]
pub fn default_auth_password_pattern() -> Regex {
    static RE: OnceCell<Regex> = OnceCell::new();

    RE.get_or_init(|| {
        Regex::new(r"(?im)(.*@.*)?password:\s?$").expect("failed compiling pattern, this is a bug")
    })
    .clone()
}

/// # Panics
///
///  Returns (once), the complied default passphrase (for private keys) pattern. This should
///  realisitcally never panic.
#[allow(clippy::expect_used)]
pub fn default_auth_passphrase_pattern() -> Regex {
    static RE: OnceCell<Regex> = OnceCell::new();

    RE.get_or_init(|| {
        Regex::new(r"(?i)enter passphrase for key")
            .expect("failed compiling pattern, this is a bug")
    })
    .clone()
}

/// # Panics
///
///  Returns (once), the complied ansi matching pattern. This should realisitcally never panic.
#[allow(clippy::expect_used)]
pub fn ansi_pattern() -> Regex {
    static RE: OnceCell<Regex> = OnceCell::new();

    RE.get_or_init(|| {
        Regex::new(
            r"[\u001B\u009B][[\\]()#;?]*(?:(?:(?:[a-zA-Z\\d]*(?:;[a-zA-Z\\d]*)*)?\u0007)|(?:(?:\\d{1,4}(?:;\\d{0,4})*)?[\\dA-PRZcf-ntqry=><~]))",
        )
        .expect("failed compiling pattern, this is a bug")
    })
    .clone()
}
