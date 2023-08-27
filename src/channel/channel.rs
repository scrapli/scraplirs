extern crate alloc;
extern crate log;
extern crate once_cell;

use crate::errors::ScrapliError;
use crate::transport::base::{
    InChannelAuthType,
    Transport,
};

use crate::util::queue::Queue;

use alloc::sync::Arc;
use log::{
    debug,
    error,
    info,
};
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{
    channel,
    Receiver,
    Sender,
};
use std::sync::Mutex;
use std::thread;

use super::Args;

/// The scraplirs `Channel` object -- the channel "wraps" the transport object and handles sending
/// and reading from the transport.
pub struct Channel {
    /// The arguments that the channel was created with.
    pub args: Args,
    pub(super) transport: Arc<Mutex<dyn Transport + Send>>,
    queue: Arc<Mutex<Queue>>,
    read_error_receiver: Option<Receiver<ScrapliError>>,
    read_done_sender: Option<Sender<bool>>,
}

impl Channel {
    /// Returns a new instance of `Channel` wrapping the given transport.
    #[must_use]
    pub fn new(
        args: Args,
        t: impl Transport + Send + 'static,
    ) -> Self {
        Self {
            args,
            transport: Arc::new(Mutex::new(t)),
            queue: Arc::new(Mutex::new(Queue::new())),
            read_error_receiver: None,
            read_done_sender: None,
        }
    }

    #[allow(clippy::significant_drop_tightening)]
    ///  Open the channel and underlying transport. This method kicks off the internal read loop
    ///  which constantly reads from the underlying transport.
    ///
    /// # Panics
    ///
    /// This method can in theory panic due to the internal queue being able to panic (but this
    /// should never happen).
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    #[allow(clippy::expect_used)]
    pub fn open(&mut self) -> Result<(), ScrapliError> {
        let Ok(mut unlocked_transport) = self.transport.lock() else {
            return Err(ScrapliError {
                details: String::from(
                    "failed acquiring transport lock during open, this should not happen",
                ),
            });
        };

        let transport_auth_data = unlocked_transport.in_channel_auth_data();

        unlocked_transport.open()?;
        drop(unlocked_transport);

        let read_loop_transport_clone = Arc::<Mutex<dyn Transport + Send>>::clone(&self.transport);
        let read_loop_queue_clone = Arc::<Mutex<Queue>>::clone(&self.queue);
        let read_delay = self.args.read_delay;

        let (read_error_sender, read_error_receiver) = channel::<ScrapliError>();
        self.read_error_receiver = Option::from(read_error_receiver);

        let (read_done_sender, read_done_receiver) = channel::<bool>();
        self.read_done_sender = Option::from(read_done_sender);

        debug!("starting channel read loop");

        thread::spawn(move || {
            Self::_read(
                &read_loop_transport_clone,
                &read_loop_queue_clone,
                read_delay,
                &read_error_sender,
                &read_done_receiver,
            );
        });

        if self.args.auth_bypass {
            debug!("auth bypass is enabled, skipping in channel auth check");

            return Ok(());
        }

        let mut auth_buff: Vec<u8> = vec![];

        match transport_auth_data.auth_type {
            InChannelAuthType::Telnet => {
                debug!("transport requests in channel telnet auth, starting...");

                auth_buff.extend(self.authenticate_telnet(
                    transport_auth_data.user.as_bytes(),
                    transport_auth_data.password.as_bytes(),
                )?);
            }

            InChannelAuthType::SSH => {
                debug!("transport requests in channel ssh auth, starting...");

                auth_buff.extend(self.authenticate_ssh(
                    transport_auth_data.password.as_bytes(),
                    transport_auth_data.private_key_passphrase.as_bytes(),
                )?);
            }
        }

        if auth_buff.is_empty() {
            return Ok(());
        }

        self.queue
            .lock()
            .expect("failed acquiring queue lock")
            .requeue(auth_buff);

        Ok(())
    }

    /// Close the channel and underlying transport.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    #[allow(clippy::expect_used)]
    pub fn close(&mut self) -> Result<(), ScrapliError> {
        info!("channel closing...");

        // send the done signal to tell our channel read loop to stop
        self.read_done_sender
            .as_ref()
            .expect("attempting to close when read done sender is not set")
            .send(true)
            .expect("error sending on read done channel, this is probably a bug");

        // not sure how to handle closing transport that may be in the middle of blocking read like
        // we do in scrapligo, so... may have to revisit this as we may never be able to acquire
        // this lock i think...
        return match self.transport.lock() {
            Ok(mut unlocked_transport) => {
                unlocked_transport.close()?;

                Ok(())
            }
            Err(err) => Err(ScrapliError {
                details: format!("failed acquiring lock on transport, error: {err}"),
            }),
        };
    }

    ///  Reads from the queue being filled by the internal (in a thread) read loop.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    ///
    /// # Panics
    ///
    /// This in theory can panic due the the basic queue implementation being able to panic,
    /// however that should not actually happen.
    #[allow(clippy::expect_used)]
    pub fn read(&mut self) -> Result<Vec<u8>, ScrapliError> {
        match self
            .read_error_receiver
            .as_ref()
            .expect("attempting to read when read error receiver is not set")
            .try_recv()
        {
            Ok(err) => {
                // there was an error in the read loop so we must propogate it up
                return Err(err);
            }
            Err(err) => {
                match err {
                    TryRecvError::Empty => {
                        // nothing received, carry on...
                    }
                    TryRecvError::Disconnected => {
                        let msg = "read error channel disconnected, this should not happen!";

                        error!("{}", msg);

                        return Err(ScrapliError {
                            details: msg.to_owned(),
                        });
                    }
                }
            }
        }

        let mut q = self.queue.lock().expect("failed acquiring queue lock");

        if q.get_depth() == 0 {
            return Ok(vec![]);
        }

        Ok(q.dequeue())
    }
}
