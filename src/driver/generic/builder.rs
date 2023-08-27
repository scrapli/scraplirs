use crate::channel::{
    Args as ChannelArgs,
    Channel,
};
use crate::driver::generic::driver::{
    Args,
    Driver,
    GenericDriverOnXCallable,
};
use crate::transport::base::{
    TransportArgs,
    TransportSSHArgs,
    TransportType,
};
use crate::transport::system::{
    System,
    SystemArgs,
};
use core::time::Duration;
use regex::bytes::Regex;

/// `Builder` is a struct that holds a bunch of settings/defaults that can be used to build a
/// *generic* Driver object.
pub struct Builder {
    args: Args,
    channel_args: ChannelArgs,
    transport_type: TransportType,
    transport_args: TransportArgs,
    transport_ssh_args: TransportSSHArgs,
    transport_system_args: SystemArgs,
}

#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::return_self_not_must_use)]
#[allow(clippy::must_use_candidate)]
impl Builder {
    /// Return a new instance of `Builder` with sane defaults set.
    pub fn new(host: &str) -> Self {
        Self {
            args: Args::new(host),
            channel_args: ChannelArgs::default(),
            transport_type: TransportType::System,
            transport_args: TransportArgs::new(host),
            transport_ssh_args: TransportSSHArgs::default(),
            transport_system_args: SystemArgs::default(),
        }
    }

    /// Sets the `auth_bypass` option -- this flag means we skip trying to do any kind of in
    /// channel authentication.
    pub fn auth_bypass(
        mut self,
        b: bool,
    ) -> Self {
        self.channel_args.auth_bypass = b;

        self
    }

    /// Sets the channel `prompt_search_depth` -- this is the depth that we search backwards in
    /// output for the prompt. Setting this smaller means we have to regex through less data but
    /// risks us "missing" the prompt which will cause us to deadlock and timeout.
    pub fn prompt_search_depth(
        mut self,
        i: u16,
    ) -> Self {
        self.channel_args.prompt_search_depth = i;

        self
    }

    /// Sets the `prompt_pattern` -- this is the primary regex pattern used by the channel to
    /// "know" when we are at a prompt (and therefore can send more data and our previous command
    /// is "done").
    pub fn prompt_pattern(
        mut self,
        r: Regex,
    ) -> Self {
        self.channel_args.prompt_pattern = r;

        self
    }

    /// Sets the `username_pattern` for use when authenticating. This is the regex pattern used to
    /// "know" when the device is prompting for the users username.
    pub fn username_pattern(
        mut self,
        r: Regex,
    ) -> Self {
        self.channel_args.username_pattern = r;

        self
    }

    /// Sets the `password_pattern` for use when authenticating. This is the regex pattern used to
    /// "know" when the device is prompting for the users password.
    pub fn password_pattern(
        mut self,
        r: Regex,
    ) -> Self {
        self.channel_args.password_pattern = r;

        self
    }

    /// Sets the `passphrase_pattern` for use when authenticating -- applicable only for *ssh*
    /// transports of course. This is the regex pattern used to "know" when the device is prompting
    /// for the ssh key passphrase.
    pub fn passphrase_pattern(
        mut self,
        r: Regex,
    ) -> Self {
        self.channel_args.passphrase_pattern = r;

        self
    }

    /// Sets the `return_char` of the channel object.
    pub fn return_char(
        mut self,
        s: &str,
    ) -> Self {
        self.channel_args.return_char = s.to_owned();

        self
    }

    /// Sets the `read_delay` of the underlying channel.
    pub fn read_delay(
        mut self,
        d: Duration,
    ) -> Self {
        self.channel_args.read_delay = d;

        self
    }

    /// Sets the `timeout_ops` of the underlying channel.
    pub fn timeout_ops(
        mut self,
        d: Duration,
    ) -> Self {
        self.channel_args.timeout_ops = d;

        self
    }

    /// Defines the transport type to use with the driver.
    pub fn transport_type(
        mut self,
        t: TransportType,
    ) -> Self {
        self.transport_type = t;

        self
    }

    /// Sets the port to connect to.
    pub fn port(
        mut self,
        i: u16,
    ) -> Self {
        self.args.port = i;
        self.transport_args.port = i;

        self
    }

    /// Sets the user(name) to use for authentication.
    pub fn user(
        mut self,
        s: &str,
    ) -> Self {
        self.transport_args.user = s.to_owned();

        self
    }

    /// Sets the password to use for authentication.
    pub fn password(
        mut self,
        s: &str,
    ) -> Self {
        self.transport_args.password = s.to_owned();

        self
    }

    /// Sets the `timeout_socket` parameter.
    pub fn timeout_socket(
        mut self,
        d: Duration,
    ) -> Self {
        self.transport_args.timeout_socket = d;

        self
    }

    /// Sets the read size of the underlying transport.
    pub fn read_size(
        mut self,
        i: u16,
    ) -> Self {
        self.transport_args.read_size = i;

        self
    }

    /// Sets the terminal height if applicable for the selected transport.
    pub fn term_height(
        mut self,
        i: u16,
    ) -> Self {
        self.transport_args.term_height = i;

        self
    }

    /// Sets the terminal width if applicable for the selected transport.
    pub fn term_width(
        mut self,
        i: u16,
    ) -> Self {
        self.transport_args.term_width = i;

        self
    }

    /// Enable or disable ssh strict key checking for *ssh* transports.
    pub fn ssh_strict_key(
        mut self,
        b: bool,
    ) -> Self {
        self.transport_ssh_args.strict_key = b;

        self
    }

    /// Sets the `private_key_path` argument of a driver using an *ssh* transport.
    pub fn ssh_private_key_path(
        mut self,
        s: &str,
    ) -> Self {
        self.transport_ssh_args.private_key_path = s.to_owned();

        self
    }

    /// Sets the `private_key_passphrase` argument of a driver using an *ssh* transport.
    pub fn ssh_private_key_passphrase(
        mut self,
        s: &str,
    ) -> Self {
        self.transport_ssh_args.private_key_passphrase = s.to_owned();

        self
    }

    /// Sets the `config_file_path` argument of a driver using an *ssh* transport.
    pub fn ssh_config_file_path(
        mut self,
        s: &str,
    ) -> Self {
        self.transport_ssh_args.config_file_path = s.to_owned();

        self
    }

    /// Sets the `known_hosts_file_path` argument of a driver using an *ssh* transport.
    pub fn ssh_known_hosts_file_path(
        mut self,
        s: &str,
    ) -> Self {
        self.transport_ssh_args.known_hosts_file_path = s.to_owned();

        self
    }

    /// Sets the `failed_when_contains` argument of a driver.
    pub fn failed_when_contains(
        mut self,
        v: Vec<String>,
    ) -> Self {
        self.args.failed_when_contains = v;

        self
    }

    /// Sets the `on_open` argument of a driver.
    pub fn on_open(
        mut self,
        f: GenericDriverOnXCallable,
    ) -> Self {
        self.args.on_open = Some(f);

        self
    }

    /// Sets the `on_close` argument of a driver.
    pub fn on_close(
        mut self,
        f: GenericDriverOnXCallable,
    ) -> Self {
        self.args.on_close = Some(f);

        self
    }

    /// Set the `open_bin` setting of a `System` transport. Will be ignored if transport type is
    /// not `System`.
    pub fn system_open_bin(
        mut self,
        s: &str,
    ) -> Self {
        self.transport_system_args.open_bin = s.to_owned();

        self
    }

    /// Set the `open_args` setting of a `System` transport. Will be ignored if transport type is
    /// not `System`.
    pub fn system_open_args(
        mut self,
        v: Vec<String>,
    ) -> Self {
        self.transport_system_args.open_args = v;

        self
    }

    /// Set the `extra_args` setting of a `System` transport. Will be ignored if transport type is
    /// not `System`.
    pub fn system_extra_args(
        mut self,
        v: Vec<String>,
    ) -> Self {
        self.transport_system_args.extra_args = v;

        self
    }

    /// Build "builds" and returns a Driver object.
    #[must_use]
    pub fn build(self) -> Driver {
        let c: Channel = match self.transport_type {
            TransportType::System => Channel::new(
                self.channel_args,
                System::new(
                    self.transport_args,
                    self.transport_ssh_args,
                    self.transport_system_args,
                ),
            ),
        };

        Driver::new(self.args, c)
    }
}
