extern crate alloc;
use super::constants::ANSI_ESCAPE_BYTE;
use super::Channel;
use crate::channel::util::strip_ansi;
use crate::errors::ScrapliError;
use crate::transport::base::Transport;
use crate::util::queue::Queue;
use alloc::sync::Arc;
use core::str;
use core::time::Duration;
use log::debug;
use std::sync::mpsc::{
    Receiver,
    Sender,
    TryRecvError,
};
use std::sync::Mutex;
use std::thread;

impl Channel {
    #[allow(clippy::expect_used)]
    pub(crate) fn _read(
        transport: &Arc<Mutex<dyn Transport + Send>>,
        queue: &Arc<Mutex<Queue>>,
        read_delay: Duration,
        read_error_sender: &Sender<ScrapliError>,
        read_done_receiver: &Receiver<bool>,
    ) {
        loop {
            match read_done_receiver.try_recv() {
                Ok(_) => {
                    // we received a done, stop reading
                    debug!("channel read loop received done signal");

                    return;
                }
                Err(err) => {
                    match err {
                        TryRecvError::Empty => {
                            // nothing received, carry on...
                        }
                        TryRecvError::Disconnected => {
                            return;
                        }
                    }
                }
            }

            let read_result = if let Ok(mut unlocked_transport) = transport.lock() {
                unlocked_transport.read()
            } else {
                read_error_sender
                    .send(ScrapliError {
                        details: String::from(
                            "failed acquiring transport lock in channel read loop",
                        ),
                    })
                    .expect("error sending on read error channel, this is probably a bug");

                thread::sleep(read_delay);

                continue;
            };

            let mut b = match read_result {
                Ok(b) => b,
                Err(err) => {
                    read_error_sender
                        .send(ScrapliError {
                            details: format!("encountered error while reading from transport in channel read loop, error: {err}"),
                        })
                        .expect("error sending on read error channel, this is probably a bug");
                    thread::sleep(read_delay);

                    continue;
                }
            };

            if !b.is_empty() {
                if b.contains(&ANSI_ESCAPE_BYTE) {
                    b = strip_ansi(&b);
                }

                debug!(
                    "channel read\n{}",
                    str::from_utf8(&b).unwrap_or("failed decoding bytes, cannot log")
                );

                let mut unlocked_queue = queue.lock().expect("failed acquiring queue lock");

                unlocked_queue.enqueue(b);

                drop(unlocked_queue);
            }

            thread::sleep(read_delay);
        }
    }
}
