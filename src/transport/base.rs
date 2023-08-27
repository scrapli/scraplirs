use crate::errors::ScrapliError;
use core::time::Duration;

/// The default port for scraplirs operations -- defaults to the standard ssh port "22".
pub const DEFAULT_PORT: u16 = 22;

/// The default time (in seconds) to use for the timeout socket parameter.
pub const DEFAULT_TIMEOUT_SOCKET_SECONDS: u64 = 30;

/// The default transport read size -- 8,192 bytes.
pub const DEFAULT_READ_SIZE: u16 = 8_192;

/// The default terminal height for transports (if applicable).
pub const DEFAULT_TERM_HEIGHT: u16 = 255;

/// The default terminal width for transports (if applicable).
pub const DEFAULT_TERM_WIDTH: u16 = 80;

/// The default ssh "strict key" setting (true, try to verify ssh key authenticity).
pub const DEFAULT_SSH_STRICT_KEY: bool = true;

/// Transport is the trait all scraplirs transports must implement in order to be consumed/used by
/// a channel and ultimately drivers.
pub trait Transport {
    /// Open the underlying transport.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if any issues occur.
    fn open(&mut self) -> Result<(), ScrapliError>;
    /// Close the underlying transport.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if any issues occur.
    fn close(&mut self) -> Result<(), ScrapliError>;
    /// Indicates if the transport is "alive".
    fn alive(&mut self) -> bool;
    /// Read default read amount of bytes from the underlying transport. Like `read_n` the
    /// implementation must be non-blocking, however this should be handled as `read` should just
    /// call `read_n` with the default (or set on transport args) number of bytes to read.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if any issues occur..
    fn read(&mut self) -> Result<Vec<u8>, ScrapliError>;
    /// Read `n` bytes from the underlying transport. Note that `read_n` implementations *must be
    /// non blocking* -- if the read for a given transport is normally non blocking, wrap it in a
    /// thread with a queue or whatever you gotta do to make sure this is not blocking!
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if any issues occur.
    fn read_n(
        &mut self,
        n: u16,
    ) -> Result<Vec<u8>, ScrapliError>;
    /// Write to the underlying transport.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if any issues occur.
    fn write(
        &mut self,
        b: &[u8],
    ) -> Result<(), ScrapliError>;
    /// Returns the `TransportArgs` of the underlying transport.
    fn get_transport_args(self) -> TransportArgs;
    /// Returns the host of the transport.
    fn get_host(&self) -> String;
    /// Returns the port of the transport.
    fn get_port(&self) -> u16;
    /// Returns info used for in channel authentication -- typically only called by the Channel.
    fn in_channel_auth_data(&self) -> InChannelAuthData;
}

/// An enum defining valid transport implementations.
pub enum TransportType {
    /// System is the "standard"/default transport implementation.
    System,
}

/// A struct hodling generic arguments that apply to all transport flavors.
pub struct TransportArgs {
    /// The actual host to connect to.
    pub host: String,
    /// The port to connect to the host on.
    pub port: u16,
    /// The username for authetnicating to the host (if applicable).
    pub user: String,
    /// The password for password or keyboard interactive authentication (if applicable).
    pub password: String,

    /// The timeout duration for initial socket connection -- see specific transports for exact
    /// implementation.
    pub timeout_socket: Duration,
    /// The read size for each read of the transport (can leave this to the default!).
    pub read_size: u16,
    /// The terminal height to set on the transport object (not applicable to all transports).
    pub term_height: u16,
    /// The terminal width to set on the transport object (not applicable to all transports).
    pub term_width: u16,
}

impl TransportArgs {
    /// Return a new instance of `TransportArgs` -- would be just a default impl but we require the
    /// host be set, so we just have this method.
    #[must_use]
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
            port: DEFAULT_PORT,
            user: String::new(),
            password: String::new(),
            timeout_socket: Duration::from_secs(DEFAULT_TIMEOUT_SOCKET_SECONDS),
            read_size: DEFAULT_READ_SIZE,
            term_height: DEFAULT_TERM_HEIGHT,
            term_width: DEFAULT_TERM_WIDTH,
        }
    }
}

/// A struct holding ssh specific arguments for transports.
pub struct TransportSSHArgs {
    /// Indicate if ssh strict key checking should be enabled or not.
    pub strict_key: bool,
    /// A path to a private key to use for authentication.
    pub private_key_path: String,
    /// An (optional) passphrase for use with a private key.
    pub private_key_passphrase: String,
    /// The path to an ssh config file to use.
    pub config_file_path: String,
    /// The path to an ssh known hosts file to use.
    pub known_hosts_file_path: String,
    /// Indicate if this is a netconf connection or not (should not be set by users).
    pub netconf_connection: bool,
}

impl Default for TransportSSHArgs {
    fn default() -> Self {
        Self {
            strict_key: DEFAULT_SSH_STRICT_KEY,
            private_key_path: String::new(),
            private_key_passphrase: String::new(),
            config_file_path: String::new(),
            known_hosts_file_path: String::new(),
            netconf_connection: false,
        }
    }
}

/// An enum indicating the type of *in channel* authentication to use for a transport.
pub enum InChannelAuthType {
    /// Telnet in channel auth -- as in we expect to see a username prompt (and no ssh pass key
    /// prompts).
    Telnet,
    /// SSH in channel auth.
    SSH,
}

/// A struct hodling data necessary for a `Channel` object to handle in channel authentication for
/// a given transport.
pub struct InChannelAuthData {
    /// Indicates the flavor of in channel authentication.
    pub auth_type: InChannelAuthType,
    /// The user to use for authenticaiton.
    pub user: String,
    /// The password to use for authentication.
    pub password: String,
    /// The ssh passphrase to use for authentication.
    pub private_key_passphrase: String,
}
