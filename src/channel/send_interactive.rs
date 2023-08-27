use super::Channel;
use super::OperationOptions;
use crate::errors::ScrapliError;
use chrono::{
    Duration as ChronoDuration,
    Utc,
};
use core::fmt;
use core::ops;
use core::str::FromStr;
use log::{
    debug,
    info,
};
use regex::bytes::Regex;

/// Holds options to use when performing "interactive" channel operations via `send_interactive`
/// `Channel` method.
pub struct Event {
    /// The input to send to the channel.
    pub input: String,
    /// The expected channel response.
    pub response: String,
    /// If the input will be "hidden" (like when entering a password).
    pub hidden: bool,
}

impl Event {
    /// Return a new instance of `SendInteractiveEvent` -- defaults to hidden being *false*.
    #[must_use]
    pub fn new(
        input: &str,
        response: &str,
    ) -> Self {
        Self {
            input: input.to_owned(),
            response: response.to_owned(),
            hidden: false,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "input: {}, expecting: {}", self.input, self.response)
    }
}

/// `SendInteractiveEvents` is a custom type for a slice of `SendInteractiveEvent` such that we can
/// implement (maybe among other future things?!) the `Display` trait.
pub struct Events(pub Vec<Event>);

impl fmt::Display for Events {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        self.0.iter().fold(Ok(()), |result, event| {
            result.and_then(|_| writeln!(f, "{event}"))
        })
    }
}

impl ops::Deref for Events {
    type Target = Vec<Event>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Channel {
    /// Send "interactive" input to the device. This is typically used to handle any well
    /// understood "interactive" prompts on a device -- things like "clear logging" which prompts
    /// the user to confirm, or handling privilege escalation where there is a password prompt.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn send_interactive(
        &mut self,
        events: &Events,
        options: &OperationOptions,
    ) -> Result<Vec<u8>, ScrapliError> {
        debug!(
            "channel send_interactive requested, processing events {}",
            events
        );

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

        let mut b: Vec<u8> = vec![];

        for (idx, event) in events.0.iter().enumerate() {
            let mut prompts = options.complete_patterns.clone();

            if event.response.is_empty() {
                prompts.push(self.args.prompt_pattern.clone());
            } else {
                let regex_response = match Regex::from_str(event.response.as_str()) {
                    Ok(r) => r,
                    Err(err) => {
                        return Err(ScrapliError {
                            details: format!(
                                "channel response '{}', could not be compiled, error: {}",
                                event.response, err
                            ),
                        })
                    }
                };

                prompts.push(regex_response);
            }

            if event.response.is_empty() {}

            self.write(event.input.as_bytes())?;

            // if the input wasn't hidden, read until we find it
            info!("reading till our input");
            if !event.input.is_empty() && !event.hidden {
                let mut rb: Vec<u8> = vec![];

                loop {
                    let now = Utc::now();

                    if deadline <= now {
                        return Err(ScrapliError {
                            details: String::from("timed out sending input to device"),
                        });
                    }

                    let (found, result) =
                        self._read_and_check_for_explicit(rb.as_ref(), event.input.as_bytes());
                    rb = match result {
                        Ok(rb) => rb,
                        Err(err) => return Err(err),
                    };

                    if rb.is_empty() {
                        continue;
                    }

                    b.extend(rb.as_slice());

                    if found {
                        break;
                    }
                }
            }

            self.write_return()?;

            // read until any prompt of prompts we set at start
            let mut rb: Vec<u8> = vec![];

            info!("return sent, reading for any prompt");
            loop {
                let now = Utc::now();

                if deadline <= now {
                    return Err(ScrapliError {
                        details: String::from("timed out sending input to device"),
                    });
                }

                let (found, result) =
                    self._read_and_check_for_any_prompt(rb.as_slice(), prompts.as_slice());

                rb = match result {
                    Ok(rb) => rb,
                    Err(err) => return Err(err),
                };

                b.extend(rb.as_slice());

                if found {
                    break;
                }
            }

            // check if we are done early based on options.complete_patterns
            if idx < events.0.len() && !options.complete_patterns.is_empty() {
                for prompt in prompts {
                    if prompt.is_match(b.as_ref()) {
                        return Ok(b);
                    }
                }
            }
        }

        Ok(b)
    }
}
