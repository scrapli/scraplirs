use crate::channel::{
    OperationOptions as ChannelOperationOptions,
    SendInteractiveEvent,
    SendInteractiveEvents,
};
use crate::driver::{
    GenericDriver,
    GenericDriverOperationOptions,
};
use crate::errors::ScrapliError;
use crate::response::{
    MultiResponse,
    Response,
};
use crate::util::strings::{
    string_contains_any_substring,
    string_vec_contains_substring,
};
use log::{
    debug,
    info,
};
use regex::bytes::{
    Regex,
    RegexBuilder,
};
use std::collections::HashMap;

const DEFAULT_CONFIGURATION_PRIVILEGE_LEVEL: &str = "configuration";

/// Note that this needs to be very high due to lots of use of char classes and obviously just
/// combining them adds to this... one day it would be nice to somehow ultra simplify things, but
/// that would be very difficult to do without potentially breaking lots of users.
const COMBINED_PROMPT_REGEX_COMPILED_BYTES_LIMIT: usize = 25_000_000;

/// The custom type for network driver on open/close callables. The `on_open` callable will be
/// executed immediately after authentication and before returning from the `open` method, while the
/// `on_close` variant will be called before closing the transport/channel. This is called *after*
/// the "generic" driver on *open* callable and *before* the generic driver on *close* callable (if
/// those are set!).
pub type NetworkDriverOnXCallable = fn(d: &Driver) -> Result<(), ScrapliError>;

/// `OperationOptions` holds arguments that apply to `Driver` operations (ex: `send_command`).
#[derive(Default, Clone)]
pub struct OperationOptions {
    /// The "generic driver" `OperationOptions` which includes the even "lower level" channel
    /// `OperationOptions`.
    pub generic_driver_operation_options: GenericDriverOperationOptions,
    /// The privilege level to execute the input in -- this only applies to `send_config`/
    /// `send_configs` methods as the `send_command`/`send_commands` methods will always acquire the
    /// `default_desired_privilege_level`.
    pub privilege_level: String,
}

/// `PrivilegeLevel` defines a privilege level, including a name, the pattern used to match a prompt
/// output to the privilege level, as well as information about how to escalate into and deescalate
/// out of this privilege level.
pub struct PrivilegeLevel {
    /// The name of the `PrivilegeLevel` ex: "exec".
    pub name: String,
    /// A regular expression pattern of the expected prompt for this `PrivilegeLevel` -- when
    /// matched and the output does not contain sub-strings from `not_contains` this means we have
    /// entered this `PrivilegeLevel`.
    pub pattern: Regex,
    /// Not contains is a vec of strings that negate a `pattern` match for this `PrivilegeLevel`.
    pub not_contains: Vec<String>,
    /// The "previous" or "lower" `PrivilegeLevel` (if exists) -- we use this to build a graph of
    /// privilege levels to move through them when `acquire_privilege_level` is called.
    pub previous_privilege_level: String,
    /// The command to "exit" or de-escalate from this `PrivilegeLevel`.
    pub de_escalate: String,
    /// The command to "enter" or escalate to this `PrivilegeLevel`.
    pub escalate: String,
    /// A  bool indicating if escalating to this `PrivilegeLevel` requires authentication -- if so,
    /// the authentication is handled by the `auth_secondary` `Arg` field.
    pub escalate_auth: bool,
    /// The prompt to expect if we have to authenticate when acquiring this `PrivilegeLevel`.
    pub escalate_prompt: String,
}

#[derive(Debug)]
enum PrivilegeAction {
    NoOp,
    Escalate,
    Deescalate,
}

/// The (network) `Driver` arguments.
pub struct Args {
    /// The "secondary" auth password (usually the "enable" password, or "sudo/root" password).
    pub secondary_password: String,
    /// The mapping of `PrivilegeLevel` for the `Driver` -- defines privilege levels such as "exec",
    /// "configuration", or "shell", etc..
    pub privilege_levels: Vec<PrivilegeLevel>,
    /// The privilege level that is considered "default" -- or that "commands" (not configs!) should
    /// be sent at -- this privilege level is acquired automatically at login and before executing
    /// any send_command(s) operations.
    pub default_desired_privilege_level: String,
    /// The "on open" callable that is executed (if set) after authenticating, and after the (if
    /// set) *generic* driver open callable is executed..
    pub(crate) on_open: Option<NetworkDriverOnXCallable>,
    /// The "on close" callable that is executed (if set) right before executing the *generic*
    /// driver close callable and before closing the channel and the underlying transport
    pub(crate) on_close: Option<NetworkDriverOnXCallable>,
}

impl Default for Args {
    /// Return a new instance of `Args` -- would be just a default impl but we require the host be
    /// set, so we just have this method.
    #[must_use]
    fn default() -> Self {
        Self {
            secondary_password: String::new(),
            privilege_levels: vec![],
            default_desired_privilege_level: String::new(),
            on_open: None,
            on_close: None,
        }
    }
}

/// Driver -- or Network Driver -- is a network driver implementation that builds on the generic
/// driver and offers some additional "network smarts" by understanding things like privilege
/// levels.
pub struct Driver {
    /// The underlying `GenericDriver`.
    pub generic_driver: GenericDriver,
    /// The `Driver` arguments (typically provided by a user or from a "platform").
    pub args: Args,

    current_privilege_level: String,
    privilege_level_graph: HashMap<String, HashMap<String, bool>>,
}

impl Driver {
    /// Create a new (network) Driver instance.
    #[must_use]
    pub fn new(
        generic_driver: GenericDriver,
        args: Args,
    ) -> Self {
        Self {
            generic_driver,
            args,
            current_privilege_level: String::new(),
            privilege_level_graph: HashMap::default(),
        }
    }

    fn build_privilege_level_graph(&mut self) {
        for privilege_level in &self.args.privilege_levels {
            let privilege_level_name = privilege_level.name.clone();

            let previous_privilege_level_name = privilege_level.previous_privilege_level.clone();

            self.privilege_level_graph
                .insert(privilege_level_name.clone(), HashMap::new());

            if privilege_level.previous_privilege_level.is_empty() {
                continue;
            }

            self.privilege_level_graph
                .entry(privilege_level_name.clone())
                .or_default()
                .insert(previous_privilege_level_name, true);
        }

        for (higher_privilege_level, privilege_level_list) in &self.privilege_level_graph.clone() {
            for privilege_level in privilege_level_list.keys() {
                self.privilege_level_graph
                    .entry(privilege_level.clone())
                    .or_default()
                    .insert(higher_privilege_level.clone(), true);
            }
        }
    }

    fn build_joined_prompt_pattern(&mut self) -> Result<(), regex::Error> {
        let joined_patterns = self
            .args
            .privilege_levels
            .iter()
            .map(|privilege_level| privilege_level.pattern.as_str())
            .collect::<Vec<&str>>()
            .join("|");

        let compiled_joined_pattern = match RegexBuilder::new(joined_patterns.as_str())
            .size_limit(COMBINED_PROMPT_REGEX_COMPILED_BYTES_LIMIT)
            .build()
        {
            Ok(compiled_joined_pattern) => compiled_joined_pattern,
            Err(err) => return Err(err),
        };

        self.generic_driver.channel.args.prompt_pattern = compiled_joined_pattern;

        Ok(())
    }

    /// Updates the network driver privilege level information -- that means this function rebuilds
    /// the internal privilege level graph and also regenerates/sets the "combined" `Channel`
    /// prompt pattern.
    ///
    /// # Errors
    ///
    /// Can error if for some reason the joined channel prompt pattern cannot be compiled.
    pub fn update_privileges(&mut self) -> Result<(), regex::Error> {
        self.build_privilege_level_graph();
        self.build_joined_prompt_pattern()
    }

    /// Open the driver and the underlying channel and transport.
    ///
    /// # Errors
    ///
    /// Can return an error if opening the underlying `generic_driver` fails. Can also return an
    /// error if the `on_open` callable is set and it returns an error.
    ///
    /// This can also return an error if (for some reason?!) the `privilege_levels` and
    /// `default_privilege_level` arguments are not set -- this should *not* happen if creating a
    /// network driver from a platform (which would be the recommended approach).
    pub fn open(&mut self) -> Result<(), ScrapliError> {
        match self.update_privileges() {
            Ok(_) => {}
            Err(err) => {
                return Err(ScrapliError {
                    details: format!(
                        "encountered error joining privilege level prompt patterns, error: {err}",
                    ),
                })
            }
        }

        if self.args.default_desired_privilege_level.is_empty()
            || self.args.privilege_levels.is_empty()
        {
            return Err(ScrapliError {
                details: String::from(
                    "default desired privilege level and/or privilege levels are unset, \
                    these are required with 'network' driver",
                ),
            });
        }

        self.generic_driver.open()?;

        if let Some(f) = self.args.on_open {
            debug!("network driver `on_open` set, executing");

            f(self)?;
        }

        Ok(())
    }

    #[allow(clippy::indexing_slicing)]
    fn determine_current_privilege_level(
        &mut self,
        current_prompt: &str,
    ) -> Result<String, ScrapliError> {
        let mut possible_current_privilege_levels: Vec<String> = vec![];

        for privilege_level in &self.args.privilege_levels {
            if string_contains_any_substring(current_prompt, privilege_level.not_contains.clone()) {
                continue;
            }

            if privilege_level.pattern.is_match(current_prompt.as_bytes()) {
                possible_current_privilege_levels.push(privilege_level.name.clone());
            }
        }

        // note that in scrapli go/py we return a slice of privs but i think we should never
        // match on more than one privilege level... so for now for rust version we'll assume that
        // anything not exactly one priv matched is an error.
        match possible_current_privilege_levels.len() {
            1 => Ok(possible_current_privilege_levels[0].clone()),
            0 => Err(ScrapliError {
                details: format!(
                    "could not determine privilege level from prompt '{current_prompt}', found *no matching privilege levels*"
                ),
            }),
            _ =>  Err(ScrapliError {
                details: format!(
                    "could not determine privilege level from prompt '{current_prompt}', found *more than one matching privilege level*"
                ),
            })
        }
    }

    #[allow(clippy::expect_used)]
    fn build_privilege_change_map(
        &self,
        current_privilege_level: &str,
        target_privilege_level: &str,
        privilege_level_steps: &Vec<String>,
    ) -> Vec<String> {
        let mut working_steps = if privilege_level_steps.is_empty() {
            vec![]
        } else {
            privilege_level_steps.clone()
        };

        working_steps.push(current_privilege_level.to_owned());

        if current_privilege_level == target_privilege_level {
            return working_steps;
        }

        for privilege_level in self
            .privilege_level_graph
            .get(current_privilege_level)
            .expect("current privilege level not found in privilege level graph, this is a bug")
            .keys()
        {
            if !string_vec_contains_substring(working_steps.clone(), privilege_level) {
                let new_working_steps = self.build_privilege_change_map(
                    privilege_level.as_str(),
                    target_privilege_level.clone(),
                    working_steps.as_ref(),
                );

                if !new_working_steps.is_empty() {
                    return new_working_steps;
                }
            }
        }

        vec![]
    }

    #[allow(clippy::indexing_slicing)]
    fn process_acquire_privilege_level(
        &mut self,
        target_privilege_level: &str,
        current_prompt: &str,
    ) -> Result<(PrivilegeAction, String), ScrapliError> {
        let current_privilege_level = self.determine_current_privilege_level(current_prompt)?;

        if current_privilege_level == target_privilege_level {
            self.current_privilege_level = current_privilege_level.clone();

            return Ok((PrivilegeAction::NoOp, current_privilege_level));
        };

        let privilege_change_map = self.build_privilege_change_map(
            current_privilege_level.as_str(),
            target_privilege_level,
            &vec![],
        );

        if privilege_change_map.is_empty() {
            return Err(ScrapliError {
                details: format!(
                    "could not build privilege level map to target privilege \
                    level '{target_privilege_level}', this is a bug"
                ),
            });
        }

        self.current_privilege_level = String::from("unknown");

        for privilege_level in &self.args.privilege_levels {
            // can't panic because zero-ith entry is always the destination priv, and we aren't in
            // the target priv, so there must be at least one more priv to go to!
            if privilege_level.name != privilege_change_map[1] {
                continue;
            }

            if privilege_level.previous_privilege_level != current_privilege_level {
                return Ok((PrivilegeAction::Deescalate, current_privilege_level));
            }

            return Ok((PrivilegeAction::Escalate, privilege_level.name.clone()));
        }

        Err(ScrapliError {
            details: format!(
                "could not determine action to take to get to privilege level \
                '{target_privilege_level}', this is a bug"
            ),
        })
    }

    /// Close the driver and the underlying channel and transport.
    ///
    /// # Errors
    ///
    /// Can return an error if closing the underlying `generic_driver` fails. Can also return an
    /// error if the `on_open` callable is set and it returns an error.
    pub fn close(&mut self) -> Result<(), ScrapliError> {
        if let Some(f) = self.args.on_close {
            debug!("network driver `on_close` set, executing");

            f(self)?;
        }

        self.generic_driver.close()
    }

    fn deescalate_privilege_level(
        &mut self,
        target_privilege_level: &str,
    ) -> Result<Vec<u8>, ScrapliError> {
        let privilege_level = match self
            .args
            .privilege_levels
            .iter()
            .find(|privilege_level| privilege_level.name == target_privilege_level)
        {
            None => {
                return Err(ScrapliError {
                    details: String::from("unknown privilege leve, this is a bug"),
                })
            }
            Some(privilege_level) => privilege_level,
        };

        self.generic_driver.channel.send_input(
            privilege_level.de_escalate.as_str(),
            &ChannelOperationOptions::default(),
        )
    }

    fn escalate_privilege_level(
        &mut self,
        target_privilege_level: &str,
    ) -> Result<Vec<u8>, ScrapliError> {
        let privilege_level = match self
            .args
            .privilege_levels
            .iter()
            .find(|privilege_level| privilege_level.name == target_privilege_level)
        {
            None => {
                return Err(ScrapliError {
                    details: String::from("unknown privilege leve, this is a bug"),
                })
            }
            Some(privilege_level) => privilege_level,
        };

        if !privilege_level.escalate_auth || self.args.secondary_password.is_empty() {
            if self.args.secondary_password.is_empty() {
                info!("no secondary password set, but escalate target may require auth, trying with no password...");
            }

            self.generic_driver.channel.send_input(
                privilege_level.escalate.as_str(),
                &ChannelOperationOptions::default(),
            )
        } else {
            let events = &SendInteractiveEvents(vec![
                SendInteractiveEvent {
                    input: privilege_level.escalate.clone(),
                    response: privilege_level.escalate_prompt.clone(),
                    hidden: false,
                },
                SendInteractiveEvent {
                    input: self.args.secondary_password.clone(),
                    response: privilege_level.pattern.to_string(),
                    hidden: true,
                },
            ]);

            self.generic_driver
                .channel
                .send_interactive(events, &ChannelOperationOptions::default())
        }
    }

    /// Acquire the target privilege level, assuming proper configuration of driver privilege levels
    /// this function will handle any escalation/de-escalate required, including entering escalation
    /// credentials (via `args.secondary_password`).
    ///
    /// # Errors
    ///
    /// Can return an error if the requested `target_privilege_level` is invalid, or a path to the
    /// target privilege level cannot be made (shouldn't happen!), or authentication into the target
    /// privilege level fails.
    #[allow(clippy::arithmetic_side_effects)]
    pub fn acquire_privilege_level(
        &mut self,
        target_privilege_level: &str,
    ) -> Result<(), ScrapliError> {
        info!(
            "acquire privilege level requested, target privilege level: {}",
            target_privilege_level
        );

        if !self
            .privilege_level_graph
            .contains_key(target_privilege_level)
        {
            return Err(ScrapliError{
                details: format!("requested privilege level '{target_privilege_level}' is not a valid privilege level"),
            });
        }

        let mut action_count: usize = 0;

        loop {
            let current_prompt = self.generic_driver.get_prompt()?;

            let (action, next_privilege_level) = self
                .process_acquire_privilege_level(target_privilege_level, current_prompt.as_str())?;

            match action {
                PrivilegeAction::NoOp => {
                    debug!("acquire privilege determined no action necessary");

                    return Ok(());
                }
                PrivilegeAction::Escalate => {
                    debug!("acquire privilege determined privilege escalation is necessary");

                    self.escalate_privilege_level(next_privilege_level.as_str())?;
                }
                PrivilegeAction::Deescalate => {
                    debug!("acquire privilege determined privilege deescalation is necessary");

                    self.deescalate_privilege_level(next_privilege_level.as_str())?;
                }
            }

            action_count += 1;

            if action_count > self.args.privilege_levels.len() * 2 {
                return Err(ScrapliError {
                    details: format!(
                        "failed to acquire target privilege level '{target_privilege_level}'"
                    ),
                });
            }
        }
    }

    /// Sends the command string to the device and returns a `Response` object. This method will
    /// always ensure that the the input is sent at the `default_desired_privilege_level`. If the
    /// current privilege level is *not* the `default_desired_privilege_level` (which is typically
    /// "privilege-exec" or "exec"), `acquire_privilege_level` will be called with a target
    /// privilege level of the `default_desired_privilege_level`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying generic driver/channel encounter an error
    /// sending the input. This function does *not* error if any `failed_when_contains` output is
    /// encountered though, *but*, the returned `Response` will indicate a failed state.
    pub fn send_command(
        &mut self,
        command: &str,
    ) -> Result<Response, ScrapliError> {
        self.send_command_with_options(command, &OperationOptions::default())
    }

    /// Sends the command string to the device and returns a `Response` object. This method will
    /// always ensure that the the input is sent at the `default_desired_privilege_level`. If the
    /// current privilege level is *not* the `default_desired_privilege_level` (which is typically
    /// "privilege-exec" or "exec"), `acquire_privilege_level` will be called with a target
    /// privilege level of the `default_desired_privilege_level`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying generic driver/channel encounter an error
    /// sending the input. This function does *not* error if any `failed_when_contains` output is
    /// encountered though, *but*, the returned `Response` will indicate a failed state.
    pub fn send_command_with_options(
        &mut self,
        command: &str,
        options: &OperationOptions,
    ) -> Result<Response, ScrapliError> {
        if self.current_privilege_level != self.args.default_desired_privilege_level {
            debug!("send_command requested but not at desired privilege level, attempting to acquire default desired privilege level");

            self.acquire_privilege_level(
                self.args.default_desired_privilege_level.clone().as_str(),
            )?;
        }

        self.generic_driver
            .send_command_with_options(command, &options.generic_driver_operation_options)
    }

    /// Sends the config lines to the device and returns a `MultiResponse` object. This method will
    /// ensure that the operation takes place in the `DEFAULT_CONFIGURATION_PRIVILEGE_LEVEL` if no
    /// privilege level is specified in the given `OperationOptions`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying generic driver/channel encounter an error
    /// sending the input. This function does *not* error if any `failed_when_contains` output is
    /// encountered though, *but*, the returned `Response` will indicate a failed state.
    pub fn send_configs(
        &mut self,
        configs: &[&str],
        options: &OperationOptions,
    ) -> Result<MultiResponse, ScrapliError> {
        let mut target_privilege_level = &options.privilege_level.as_str();

        if target_privilege_level.is_empty() {
            target_privilege_level = &DEFAULT_CONFIGURATION_PRIVILEGE_LEVEL;
        }

        self.acquire_privilege_level(target_privilege_level)?;

        self.generic_driver
            .send_commands_with_options(configs, &options.generic_driver_operation_options)
    }
}
