extern crate nix;
use crate::errors::ScrapliError;
use crate::transport::base::{
    InChannelAuthData,
    InChannelAuthType,
    Transport,
    TransportArgs,
    TransportSSHArgs,
};
use crate::util::ptyprocess::PtyProcess;
use log::debug;
use nix::poll::{
    poll,
    PollFd,
    PollFlags,
};
use nix::sys::wait::WaitStatus;
use nix::unistd::dup;
use std::fs::File;
use std::io::{
    BufReader,
    BufWriter,
    Read,
    Write,
};
use std::os::fd::RawFd;
use std::os::unix::io::{
    AsRawFd,
    FromRawFd,
};
use std::process::Command;

/// The default binary to use for the `System` transport -- "ssh".
pub const DEFAULT_SSH_OPEN_BIN: &str = "ssh";

/// A struct holding arguments specific to the `System` transport implementation.
#[allow(clippy::module_name_repetitions)]
pub struct SystemArgs {
    /// The actual name of the binary to use to open the `System` transport -- typically this is
    /// "ssh", but you could do things like "docker" or "kubectl" (for exec operations) instead.
    pub open_bin: String,
    /// Arguments to pass to the `open_bin` -- if unset/empty "normal" ssh options will be set based
    /// on the arguments provided to the transport.
    pub open_args: Vec<String>,
    /// Extra arguments to pass -- so you can pass any ssh flags in addition to the "normal" ssh
    /// options set based on the arguments provided to the transport.
    pub extra_args: Vec<String>,
}

impl Default for SystemArgs {
    fn default() -> Self {
        Self {
            open_bin: String::from(DEFAULT_SSH_OPEN_BIN),
            open_args: vec![],
            extra_args: vec![],
        }
    }
}

/// The "system" (/bin/ssh, or "original" scrapli) transport object.
pub struct System {
    args: TransportArgs,
    ssh_args: TransportSSHArgs,
    system_args: SystemArgs,
    process: Option<PtyProcess>,
    file: Option<File>,
    file_handle: RawFd,
    reader: Option<BufReader<File>>,
    writer: Option<BufWriter<File>>,
}

impl System {
    /// Returns a new `System` instance.
    #[must_use]
    pub const fn new(
        args: TransportArgs,
        ssh_args: TransportSSHArgs,
        system_args: SystemArgs,
    ) -> Self {
        Self {
            args,
            ssh_args,
            system_args,
            process: None,
            file: None,
            file_handle: -1,
            reader: None,
            writer: None,
        }
    }

    fn build_open_args(&mut self) {
        if !self.system_args.open_args.is_empty() {
            self.system_args.open_args = vec![];
        }

        self.system_args.open_args = vec![
            self.args.host.clone(),
            String::from("-p"),
            format!("{}", self.args.port),
            String::from("-o"),
            format!("ConnectTimeout={}", self.args.timeout_socket.as_secs()),
            String::from("-o"),
            format!("ServerAliveInterval={}", self.args.timeout_socket.as_secs()),
        ];

        if !self.args.user.is_empty() {
            self.system_args
                .open_args
                .extend([String::from("-l"), self.args.user.clone()]);
        }

        if self.ssh_args.strict_key {
            self.system_args.open_args.extend([
                String::from("-o"),
                String::from("StrictHostKeyChecking=yes"),
            ]);

            if !self.ssh_args.known_hosts_file_path.is_empty() {
                self.system_args.open_args.extend([
                    String::from("-o"),
                    format!("UserKnownHostsFile={}", self.ssh_args.known_hosts_file_path),
                ]);
            }
        } else {
            self.system_args.open_args.extend([
                String::from("-o"),
                String::from("StrictHostKeyChecking=no"),
                String::from("-o"),
                String::from("UserKnownHostsFile=/dev/null"),
            ]);
        }

        if !self.ssh_args.config_file_path.is_empty() {
            self.system_args
                .open_args
                .extend([String::from("-F"), self.ssh_args.config_file_path.clone()]);
        }

        if !self.ssh_args.private_key_path.is_empty() {
            self.system_args
                .open_args
                .extend([String::from("-i"), self.ssh_args.private_key_path.clone()]);
        }

        if !self.system_args.extra_args.is_empty() {
            self.system_args
                .open_args
                .extend(self.system_args.extra_args.clone());
        }
    }

    fn setup_reader_writer(&mut self) -> Result<(), ScrapliError> {
        let mut open_cmd = Command::new(self.system_args.open_bin.clone());
        open_cmd.args(self.system_args.open_args.clone());

        let process = match PtyProcess::new(open_cmd) {
            Ok(process) => process,
            Err(err) => {
                return Err(ScrapliError {
                    details: format!("encountered error spawning pty process, error: {err}"),
                })
            }
        };

        let fd = match dup(process.pty.as_raw_fd()) {
            Ok(fd) => fd,
            Err(err) => {
                return Err(ScrapliError {
                    details: format!(
                        "encountered error duplicated pty process file handle, error: {err}"
                    ),
                })
            }
        };

        self.process = Some(process);

        // SAFETY: the file descriptor must be valid!
        let file = unsafe { File::from_raw_fd(fd) };

        let writer_clone = match file.try_clone() {
            Ok(writer_clone) => writer_clone,
            Err(err) => {
                return Err(ScrapliError {
                    details: format!(
                        "failed cloning pty file handle for writer object, error: {err}"
                    ),
                })
            }
        };

        self.writer = Option::from(BufWriter::new(writer_clone));

        let reader_clone = match file.try_clone() {
            Ok(reader_clone) => reader_clone,
            Err(err) => {
                return Err(ScrapliError {
                    details: format!(
                        "failed cloning pty file handle for reader object, error: {err}"
                    ),
                })
            }
        };

        self.reader = Option::from(BufReader::new(reader_clone));

        self.file_handle = file.as_raw_fd();
        self.file = Some(file);

        Ok(())
    }
}

impl Transport for System {
    fn open(&mut self) -> Result<(), ScrapliError> {
        if self.system_args.open_args.is_empty() {
            self.build_open_args();
        }

        debug!(
            "opening system transport with bin '{}' and args '{:?}'",
            self.system_args.open_bin, self.system_args.open_args
        );

        self.setup_reader_writer()?;

        Ok(())
    }

    fn close(&mut self) -> Result<(), ScrapliError> {
        let process = match self.process.as_mut() {
            None => {
                return Err(ScrapliError {
                    details: String::from("trying to close transport with no process created"),
                })
            }
            Some(process) => process,
        };

        match process.exit() {
            Ok(_) => Ok(()),
            Err(err) => Err(ScrapliError {
                details: format!("failed closing pty process, error: {err}"),
            }),
        }
    }

    fn alive(&mut self) -> bool {
        self.process.as_mut().map_or(false, |process| {
            process.status().map_or(false, |status| {
                matches!(status, WaitStatus::Continued(_) | WaitStatus::StillAlive)
            })
        })
    }

    fn read(&mut self) -> Result<Vec<u8>, ScrapliError> {
        self.read_n(self.args.read_size)
    }

    /// Read up to `n` bytes from the transport.
    ///
    /// Allows `indexing_slicing` since we explicitly create the byte slice we read into and we know
    /// we can never read more bytes than we allocated. Therefore, when we slice out the null bytes
    /// we know that is a safe operation.
    #[allow(clippy::indexing_slicing)]
    #[allow(clippy::significant_drop_tightening)]
    fn read_n(
        &mut self,
        n: u16,
    ) -> Result<Vec<u8>, ScrapliError> {
        let fd = PollFd::new(self.file_handle, PollFlags::POLLIN);

        match poll(&mut [fd], 5) {
            Ok(r) => {
                if r != 1 {
                    return Ok(vec![]);
                }
            }
            Err(err) => {
                return Err(ScrapliError {
                    details: format!("error while polling fd, error: {err}"),
                })
            }
        }

        let mut b = vec![0_u8; n as usize];

        let reader = match self.reader {
            None => {
                return Err(ScrapliError {
                    details: String::from("attempting to read from transport with no process!"),
                })
            }
            Some(ref mut reader) => reader,
        };

        return match reader.read(b.as_mut_slice()) {
            Ok(read_n) => Ok(b[0..read_n].to_owned()),
            Err(err) => Err(ScrapliError {
                details: format!("error when reading after polling fd, error: {err}"),
            }),
        };
    }

    fn write(
        &mut self,
        b: &[u8],
    ) -> Result<(), ScrapliError> {
        let writer = match self.writer {
            None => {
                return Err(ScrapliError {
                    details: String::from("attempting to write to transport with no process!"),
                })
            }
            Some(ref mut writer) => writer,
        };

        match writer.write_all(b) {
            Ok(_) => {}
            Err(err) => {
                return Err(ScrapliError {
                    details: format!("failed writing to transport, error: {err}"),
                })
            }
        };

        match writer.flush() {
            Ok(_) => Ok(()),
            Err(err) => Err(ScrapliError {
                details: format!("failed flushing transport, error: {err}"),
            }),
        }
    }

    fn get_transport_args(self) -> TransportArgs {
        self.args
    }

    fn get_host(&self) -> String {
        self.args.host.clone()
    }

    fn get_port(&self) -> u16 {
        self.args.port
    }

    fn in_channel_auth_data(&self) -> InChannelAuthData {
        InChannelAuthData {
            auth_type: InChannelAuthType::SSH,
            user: self.args.user.clone(),
            password: self.args.password.clone(),
            private_key_passphrase: self.ssh_args.private_key_passphrase.clone(),
        }
    }
}
