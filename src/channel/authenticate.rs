use super::constants::{
    PASSPHRASE_SEEN_MAX,
    PASSWORD_SEEN_MAX,
    USER_SEEN_MAX,
};
use super::Channel;
use crate::channel::patterns::{
    default_auth_passphrase_pattern,
    default_auth_password_pattern,
    default_auth_username_pattern,
};
use crate::errors::ScrapliError;
use log::error;

impl Channel {
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn authenticate_telnet(
        &mut self,
        user: &[u8],
        password: &[u8],
    ) -> Result<Vec<u8>, ScrapliError> {
        let mut user_seen_count = 0;
        let mut password_seen_count = 0;

        let mut rb: Vec<u8> = vec![];

        loop {
            let nb = self.read_until_any_prompt(&[
                self.args.prompt_pattern.clone(),
                self.args.username_pattern.clone(),
                self.args.password_pattern.clone(),
            ])?;

            if nb.is_empty() {
                continue;
            }

            rb.extend(nb);

            if self.args.prompt_pattern.is_match(&rb) {
                return Ok(rb);
            }

            if default_auth_username_pattern().is_match(&rb) {
                user_seen_count += 1;

                if user_seen_count > USER_SEEN_MAX {
                    let msg = String::from(
                        "user prompt seen multiple times, assuming authentication failed",
                    );

                    error!("{}", msg);

                    return Err(ScrapliError { details: msg });
                }

                self.write_and_return(user)?;

                rb = vec![];

                continue;
            }

            if default_auth_password_pattern().is_match(&rb) {
                password_seen_count += 1;

                if password_seen_count > PASSWORD_SEEN_MAX {
                    let msg = String::from(
                        "password prompt seen multiple times, assuming authentication failed",
                    );

                    error!("{}", msg);

                    return Err(ScrapliError { details: msg });
                }

                self.write_and_return(password)?;

                rb = vec![];
            }
        }
    }

    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn authenticate_ssh(
        &mut self,
        password: &[u8],
        passphrase: &[u8],
    ) -> Result<Vec<u8>, ScrapliError> {
        let mut password_seen_count = 0;
        let mut passphrase_seen_count = 0;

        let mut rb: Vec<u8> = vec![];

        loop {
            let nb = self.read_until_any_prompt(&[
                self.args.prompt_pattern.clone(),
                self.args.password_pattern.clone(),
                self.args.passphrase_pattern.clone(),
            ])?;

            if nb.is_empty() {
                continue;
            }

            rb.extend(nb);

            if self.args.prompt_pattern.is_match(&rb) {
                return Ok(rb);
            }

            if default_auth_password_pattern().is_match(&rb) {
                password_seen_count += 1;

                if password_seen_count > PASSWORD_SEEN_MAX {
                    let msg = String::from(
                        "password prompt seen multiple times, assuming authentication failed",
                    );

                    error!("{}", msg);

                    return Err(ScrapliError { details: msg });
                }

                self.write_and_return(password)?;

                rb = vec![];

                continue;
            }

            if default_auth_passphrase_pattern().is_match(&rb) {
                passphrase_seen_count += 1;

                if passphrase_seen_count > PASSPHRASE_SEEN_MAX {
                    let msg = String::from("private key passphrase prompt seen multiple times, assuming authentication failed");

                    error!("{}", msg);

                    return Err(ScrapliError { details: msg });
                }

                self.write_and_return(passphrase)?;

                rb = vec![];
            }
        }
    }
}
