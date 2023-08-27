//! Start a process via pty

/// rexpect / `PtyProcess` was created and licensed by Philipp Keller, this module is a vendored and
/// lightly modified version of their work, the following is a copy of the rexpect cates license
/// file. thank you to those folks excellent work!
///
/// MIT License
///
/// Copyright (c) 2018 Philipp Keller
///
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// The above copyright notice and this permission notice shall be included in all
/// copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.
extern crate errno;
use core;
use nix;
use nix::fcntl;
use nix::fcntl::{
    fcntl,
    open,
    OFlag,
};
use nix::libc::{
    STDERR_FILENO,
    STDIN_FILENO,
    STDOUT_FILENO,
};
use nix::pty::{
    grantpt,
    posix_openpt,
    unlockpt,
    PtyMaster,
};
use nix::sys::{
    signal,
    wait,
};
use nix::sys::{
    stat,
    termios,
};
use nix::unistd::{
    dup2,
    fork,
    setsid,
    ForkResult,
    Pid,
};
use std::io::Error;
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::{
    thread,
    time,
};

#[derive(Debug, thiserror::Error)]
/// Vendored error object from rexpect, wraps other errors nicely.
pub enum PtyProcessError {
    #[error(transparent)]
    /// Wrapper around nix errors.
    Nix(#[from] nix::Error),

    #[error(transparent)]
    /// Wrapper around std::io errors.
    Io(#[from] Error),

    #[cfg(feature = "which")]
    #[error(transparent)]
    Which(#[from] which::Error),
}

/// Start a process in a forked tty so you can interact with it the same as you would within a
/// a terminal.
///
/// The process and pty session are killed upon dropping `PtyProcess`.
pub struct PtyProcess {
    /// The actual pty object.
    pub pty: PtyMaster,
    child_pid: Pid,
    kill_timeout: Option<time::Duration>,
}

#[cfg(target_os = "linux")]
use nix::pty::ptsname_r;

#[cfg(target_os = "macos")]
/// `ptsname_r` is a linux extension but ptsname isn't thread-safe instead of using a static mutex
/// this calls ioctl with TIOCPTYGNAME directly based on
/// <https://blog.tarq.io/ptsname-on-osx-with-rust/>
fn ptsname_r(fd: &PtyMaster) -> nix::Result<String> {
    use core::ffi::CStr;
    use nix::libc::{
        ioctl,
        TIOCPTYGNAME,
    };

    // the buffer size on OSX is 128, defined by sys/ttycom.h
    let mut buf: [i8; 128] = [0; 128];

    // SAFETY: ioctl is unsafe for... reasons.
    // SAFETY: the buf as a pointer must be valid or this is unsafe (maybe among other things...).
    unsafe {
        match ioctl(fd.as_raw_fd(), u64::from(TIOCPTYGNAME), &mut buf) {
            0_i32 => {
                let res = CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned();
                Ok(res)
            }
            _ => Err(nix::Error::last()),
        }
    }
}

impl PtyProcess {
    /// Start a process in a forked pty
    ///
    /// # Errors
    ///
    /// Returns a `PtyProcessError` if the flags cannot be set properly or file handles cannot be
    /// duplicated, or generally if anything unrecoverable happens.
    pub fn new(mut command: Command) -> Result<Self, PtyProcessError> {
        const APPLY_NONBLOCK_AFTER_OPEN: bool = cfg!(target_os = "freebsd");

        // Open a new PTY master
        let master_fd = if APPLY_NONBLOCK_AFTER_OPEN {
            posix_openpt(OFlag::O_RDWR | OFlag::O_NOCTTY)?
        } else {
            posix_openpt(OFlag::O_RDWR | OFlag::O_NOCTTY | OFlag::O_NONBLOCK)?
        };

        // Allow a slave to be generated for it
        grantpt(&master_fd)?;
        unlockpt(&master_fd)?;

        if APPLY_NONBLOCK_AFTER_OPEN {
            let raw_fd = master_fd.as_raw_fd();
            let flags = fcntl(raw_fd, fcntl::F_GETFL)?;
            if flags < 0_i32 {
                return Err(PtyProcessError::from(Error::last_os_error()));
            }

            let flag_bits = match OFlag::from_bits(flags) {
                None => return Err(PtyProcessError::from(nix::Error::UnknownErrno)),
                Some(flag_bits) => flag_bits,
            };

            if fcntl(raw_fd, fcntl::F_SETFL(flag_bits | OFlag::O_NONBLOCK)) == Ok(-1_i32) {
                return Err(PtyProcessError::from(Error::last_os_error()));
            }
        }

        // on Linux this is the libc function, on OSX this is our implementation of ptsname_r
        let slave_name = ptsname_r(&master_fd)?;

        // SAFETY: only async-signal-safe functions should be called from the fork.
        match unsafe { fork()? } {
            ForkResult::Child => {
                setsid()?; // create new session with child as session leader
                let slave_fd = open(
                    std::path::Path::new(&slave_name),
                    OFlag::O_RDWR,
                    stat::Mode::empty(),
                )?;

                // assign stdin, stdout, stderr to the tty, just like a terminal does
                dup2(slave_fd, STDIN_FILENO)?;
                dup2(slave_fd, STDOUT_FILENO)?;
                dup2(slave_fd, STDERR_FILENO)?;

                // set echo off
                let mut flags = termios::tcgetattr(STDIN_FILENO)?;
                flags.local_flags &= !termios::LocalFlags::ECHO;
                termios::tcsetattr(STDIN_FILENO, termios::SetArg::TCSANOW, &flags)?;

                command.exec();
                Err(PtyProcessError::Nix(nix::Error::last()))
            }
            ForkResult::Parent { child: child_pid } => Ok(Self {
                pty: master_fd,
                child_pid,
                kill_timeout: None,
            }),
        }
    }

    /// At the drop of `PtyProcess` the running process is killed. This is blocking forever if the
    /// process does not react to a normal kill. If `kill_timeout` is set the process is
    /// `kill -9`ed after duration.
    #[allow(dead_code)]
    pub fn set_kill_timeout(
        &mut self,
        timeout_ms: Option<u64>,
    ) {
        self.kill_timeout = timeout_ms.map(time::Duration::from_millis);
    }

    /// Get status of child process, non-blocking.
    ///
    /// This method runs waitpid on the process. This means: If you ran `exit()` before or
    /// `status()` this method will return `None`.
    #[must_use]
    #[allow(clippy::option_if_let_else)]
    pub fn status(&self) -> Option<wait::WaitStatus> {
        let status_result = wait::waitpid(self.child_pid, Some(wait::WaitPidFlag::WNOHANG));

        match status_result {
            Ok(status) => Some(status),
            Err(_) => None,
        }
    }

    /// Wait until process has exited. This is a blocking call. If the process doesn't terminate
    /// this will block forever.
    ///
    /// # Errors
    ///
    /// Returns a `PtyProcessError` if the wait fails.
    #[allow(dead_code)]
    pub fn wait(&self) -> Result<wait::WaitStatus, PtyProcessError> {
        wait::waitpid(self.child_pid, None).map_err(PtyProcessError::from)
    }

    /// Regularly exit the process, this method is blocking until the process is dead
    ///
    /// # Errors
    ///
    /// Returns a `PtyProcessError` if the process cannot be killed.
    pub fn exit(&mut self) -> Result<wait::WaitStatus, PtyProcessError> {
        self.kill(signal::SIGTERM).map_err(PtyProcessError::from)
    }

    /// Non-blocking variant of `kill()` (doesn't wait for process to be killed)
    ///
    /// # Errors
    ///
    /// Returns a `PtyProcessError` if the process cannot be signaled to stop.
    #[allow(dead_code)]
    pub fn signal(
        &mut self,
        sig: signal::Signal,
    ) -> Result<(), PtyProcessError> {
        signal::kill(self.child_pid, sig).map_err(PtyProcessError::from)
    }

    /// Kill the process with a specific signal. This method blocks, until the process is dead.
    ///
    /// Repeatedly sends SIGTERM to the process until it died, the pty session is closed upon
    /// dropping `PtyMaster`, so we don't need to explicitly do that here.
    ///
    /// If `kill_timeout` is set and a repeated sending of signal does not result in the process
    /// being killed, then `kill -9` is sent after the `kill_timeout` duration has elapsed.
    ///
    /// # Errors
    ///
    /// Returns a `PtyProcessError` if the process cannot be killed.
    pub fn kill(
        &mut self,
        sig: signal::Signal,
    ) -> Result<wait::WaitStatus, PtyProcessError> {
        let start = time::Instant::now();
        loop {
            match signal::kill(self.child_pid, sig) {
                Ok(_) => {}
                // process was already killed before -> ignore
                Err(nix::errno::Errno::ESRCH) => {
                    return Ok(wait::WaitStatus::Exited(Pid::from_raw(0), 0))
                }
                Err(e) => return Err(PtyProcessError::from(e)),
            }

            match self.status() {
                Some(status) if status != wait::WaitStatus::StillAlive => return Ok(status),
                Some(_) | None => thread::sleep(time::Duration::from_millis(100)),
            }
            // kill -9 if timout is reached
            if let Some(timeout) = self.kill_timeout {
                if start.elapsed() > timeout {
                    signal::kill(self.child_pid, signal::Signal::SIGKILL)
                        .map_err(PtyProcessError::from)?;
                }
            }
        }
    }
}

impl Drop for PtyProcess {
    fn drop(&mut self) {
        if self.status() == Some(wait::WaitStatus::StillAlive) {
            #[allow(clippy::expect_used)]
            self.exit().expect("cannot exit");
        }
    }
}
