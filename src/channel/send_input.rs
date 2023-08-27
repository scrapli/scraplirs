use super::constants::NEW_LINE_BYTE;
use super::Channel;
use super::OperationOptions;
use crate::errors::ScrapliError;
use crate::util::bytes::{
    trim_cutset,
    trim_cutset_right,
};
use chrono::{
    Duration as ChronoDuration,
    Utc,
};
use std::thread;

impl Channel {
    #[allow(clippy::indexing_slicing)]
    fn process_output(
        &self,
        b: &[u8],
        strip_prompt: bool,
    ) -> Vec<u8> {
        let lines = b.split(|b| b == &NEW_LINE_BYTE);

        let mut clean_lines = vec![vec![0_u8]; lines.clone().count()];

        for (idx, mut line) in lines.into_iter().enumerate() {
            line = trim_cutset_right(line, &[NEW_LINE_BYTE]);

            clean_lines[idx] = [line, &[NEW_LINE_BYTE]].concat();
        }

        let mut joined_lines = clean_lines.concat();

        if strip_prompt {
            joined_lines = self
                .args
                .prompt_pattern
                .replace(joined_lines.as_slice(), vec![])
                .to_vec();
        }

        // trim any remaining newlines left/right and also the return character
        let mut cutset = vec![NEW_LINE_BYTE];
        cutset.extend(self.args.return_char.as_bytes());

        let joined_cleaned_lines = trim_cutset(joined_lines.as_slice(), cutset.as_slice());

        joined_cleaned_lines.to_vec()
    }

    /// Send an input to the device.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn send_input_bytes(
        &mut self,
        b: &[u8],
        options: &OperationOptions,
    ) -> Result<Vec<u8>, ScrapliError> {
        let timeout = match ChronoDuration::from_std(options.timeout.unwrap_or(self.args.timeout_ops)) {
            Ok(timeout) => timeout,
            Err(err) => {
                return Err(
                    ScrapliError{
                        details: format!("failed casting std Duration to chrono Duration, this shouldn't happen, error: {err}")
                    }
                )
            }
        };

        let deadline = Utc::now() + timeout;

        self.write(b)?;

        let mut rb: Vec<u8> = vec![];

        loop {
            let now = Utc::now();

            if deadline <= now {
                return Err(ScrapliError {
                    details: String::from("timed out sending input to device"),
                });
            }

            let (found, result) = self._read_and_check_for_fuzzy(rb.as_slice(), b);
            rb = match result {
                Ok(rb) => rb,
                Err(err) => return Err(err),
            };

            if found {
                break;
            }
        }

        self.write_return()?;

        if options.eager {
            return Ok(b.to_vec());
        }

        let mut rb: Vec<u8> = vec![];

        loop {
            let now = Utc::now();

            if deadline <= now {
                return Err(ScrapliError {
                    details: String::from("timed out sending input to device"),
                });
            }

            let (found, result): (bool, Result<Vec<u8>, ScrapliError>);

            if options.interim_prompt_patterns.is_empty() {
                (found, result) = self._read_and_check_for_prompt(rb.as_slice());
            } else {
                // read until any prompt (need to make new method like the check kind below)
                // where prompt is any normal prompts ++ the interim prompt patterns
                (found, result) = self._read_and_check_for_any_prompt(
                    rb.as_slice(),
                    options.interim_prompt_patterns.as_slice(),
                );
            }

            rb = match result {
                Ok(rb) => rb,
                Err(err) => return Err(err),
            };

            if found {
                return Ok(self.process_output(rb.as_slice(), options.strip_prompt));
            }

            // to not just totally slam cpu, some very unscientific testing indicates this feels
            // like a decent mix of not slamming cpu while not sleeping too long... in theory if
            // some user decided to set the read delay to like a zillion this could be bad but then
            // again that would make everything pretty bad anyway :)
            thread::sleep(self.args.read_delay / 8);
        }
    }

    /// Send an input to the device, this is a convenience function to write a string, it wraps
    /// `send_input_bytes`.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn send_input(
        &mut self,
        input: &str,
        options: &OperationOptions,
    ) -> Result<Vec<u8>, ScrapliError> {
        self.send_input_bytes(input.as_bytes(), options)
    }
}
