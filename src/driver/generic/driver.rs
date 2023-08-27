use crate::channel::Channel;
use crate::channel::OperationOptions as ChannelOperationOptions;
use crate::errors::ScrapliError;
use crate::response::{
    MultiResponse,
    Response,
};
use crate::transport::base::DEFAULT_PORT;
use log::{
    debug,
    info,
};

/// The custom type for generic driver on open/close callables. The `on_open` callable will be
/// executed immediately after authentication and before returning from the `open` method, while the
/// `on_close` variant will be called before closing the transport/channel.
pub type GenericDriverOnXCallable = fn(d: &Driver) -> Result<(), ScrapliError>;

/// `OperationOptions` holds arguments that apply to `Driver` operations (ex: `send_command`).
#[derive(Default, Clone)]
pub struct OperationOptions {
    /// List of strings that when seen as sub strings in some output indicate that the operation was
    /// a failure.
    pub failed_when_contains: Vec<String>,
    /// Indicates if multi operations (send_commands (plural!)) that encounter a failure (based on
    /// `failed_when_contains` output) should stop or not.
    pub stop_on_failed: bool,
    /// Channel operation options that are passed (by the driver) down to the channel during normal
    /// operations.
    pub channel_operation_options: ChannelOperationOptions,
}

/// Args are standard driver args that will be stored with a driver object -- the host and port will
/// be automatically copied from the transport if using normal builder paths.
pub struct Args {
    /// The host the driver is connecting to.
    pub host: String,
    /// The port on the host the driver is connecting to.
    pub port: u16,
    /// The list of strings which indicate command failures.
    pub failed_when_contains: Vec<String>,
    /// The "on open" callable that is executed (if set) immediately after authenticating.
    pub(crate) on_open: Option<GenericDriverOnXCallable>,
    /// The "on close" callable that is executed (if set) right before closing the channel and the
    /// underlying transport.
    pub(crate) on_close: Option<GenericDriverOnXCallable>,
}

impl Args {
    /// Return a new instance of `Args` -- would be just a default impl but we require the host be
    /// set, so we just have this method.
    #[must_use]
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
            port: DEFAULT_PORT,
            failed_when_contains: vec![],
            on_open: None,
            on_close: None,
        }
    }
}

/// Driver -- or Generic Driver -- is a generic driver implementation that offers some basic methods
/// for interacting with a device. A (generic) Driver knows nothing about network-y things like
/// privilege levels and the like, and is more of a fancier expect-like interface.
pub struct Driver {
    /// The standard driver args.
    pub args: Args,
    /// The channel the driver interacts with.
    pub channel: Channel,
}

impl Driver {
    /// Create a new (generic) Driver instance.
    #[must_use]
    pub const fn new(
        args: Args,
        channel: Channel,
    ) -> Self {
        Self { args, channel }
    }

    /// Open the driver and the underlying channel and transport.
    ///
    /// # Errors
    ///
    /// Can return an error if opening the channel fails. Can also return an error if the `on_open`
    /// callable is set and it returns an error.
    pub fn open(&mut self) -> Result<(), ScrapliError> {
        debug!(
            "opening connection to host {} on port {}",
            self.args.host, self.args.port
        );

        self.channel.open()?;

        if let Some(f) = self.args.on_open {
            debug!("generic driver `on_open` set, executing");

            f(self)?;
        }

        info!("connection opened successfully");

        Ok(())
    }

    /// Close the driver and the underlying channel and transport.
    ///
    /// # Errors
    ///
    /// Can return an error if closing the channel fails. Can also return an error if the `on_close`
    /// callable is set and it returns an error.
    pub fn close(&mut self) -> Result<(), ScrapliError> {
        debug!(
            "closing connection to host {} on port {}",
            self.args.host, self.args.port
        );

        if let Some(f) = self.args.on_open {
            debug!("generic driver `on_close` set, executing");

            f(self)?;
        }

        self.channel.close()?;

        info!("connection closed successfully");

        Ok(())
    }

    /// Return the current "prompt" from the device.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying channel errored on the `get_prompt` call.
    ///
    /// # Panics
    ///
    /// Can panic if there is invalid utf-8 in the bytes in prompt byte vec returned from the
    /// channel.
    #[allow(clippy::expect_used)]
    pub fn get_prompt(&mut self) -> Result<String, ScrapliError> {
        match self.channel.get_prompt() {
            Ok(prompt_bytes) => {
                Ok(String::from_utf8(prompt_bytes).expect("invalid utf-8 in prompt"))
            }
            Err(err) => Err(ScrapliError {
                details: format!("error fetching prompt from channel, error: {err}"),
            }),
        }
    }

    /// Send a command to the device.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn send_command(
        &mut self,
        command: &str,
    ) -> Result<Response, ScrapliError> {
        let opts = &mut OperationOptions::default();
        opts.failed_when_contains = self.args.failed_when_contains.clone();

        self.send_command_with_options(command, opts)
    }

    /// Send a command to the device with optional options struct provided.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn send_command_with_options(
        &mut self,
        command: &str,
        options: &OperationOptions,
    ) -> Result<Response, ScrapliError> {
        info!("send_command requested, sending '{}'", command);

        let opts = &mut options.clone();

        if options.failed_when_contains.is_empty() {
            opts.failed_when_contains = self.args.failed_when_contains.clone();
        }

        let mut resp = Response::new(
            command,
            self.args.host.as_str(),
            self.args.port,
            opts.failed_when_contains.clone(),
        );

        match self
            .channel
            .send_input(command, &opts.channel_operation_options)
        {
            Ok(rb) => {
                resp.record(rb);

                Ok(resp)
            }
            Err(err) => Err(err),
        }
    }

    /// Send a list of commands to the device.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn send_commands(
        &mut self,
        commands: &[&str],
    ) -> Result<MultiResponse, ScrapliError> {
        let opts = &mut OperationOptions::default();
        opts.failed_when_contains = self.args.failed_when_contains.clone();

        self.send_commands_with_options(commands, opts)
    }

    /// Send a list of commands to the device.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    #[allow(clippy::indexing_slicing)]
    pub fn send_commands_with_options(
        &mut self,
        commands: &[&str],
        options: &OperationOptions,
    ) -> Result<MultiResponse, ScrapliError> {
        if commands.is_empty() {
            return Err(ScrapliError {
                details: String::from("send_commands called with empty vec of commands"),
            });
        }

        info!("send_commands requested, sending '{:?}'", commands);

        let mut multi_response = MultiResponse::new(self.args.host.as_str());

        for command in &commands[..commands.len() - 1] {
            let response = self.send_command_with_options(command, options)?;

            let failed = response.failed;

            multi_response.record_response(response);

            if options.stop_on_failed && failed {
                info!("stop on failed is true and a command failed, discontinuing send commands operation");

                return Ok(multi_response);
            }
        }

        let final_response =
            self.send_command_with_options(commands[commands.len() - 1], options)?;

        multi_response.record_response(final_response);

        Ok(multi_response)
    }
}
