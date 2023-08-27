use super::Channel;
use crate::errors::ScrapliError;

impl Channel {
    /// Write `b` bytes to the device -- typically you should use `write_and_return` instead.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn write(
        &mut self,
        b: &[u8],
    ) -> Result<(), ScrapliError> {
        return match self.transport.lock() {
            Ok(mut unlocked_transport) => {
                unlocked_transport.write(b)?;

                Ok(())
            }
            Err(err) => Err(ScrapliError {
                details: format!("failed acquiring lock on transport, error: {err}"),
            }),
        };
    }

    /// Writes a return -- the return character by default is "\n", but can be configured.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn write_return(&mut self) -> Result<(), ScrapliError> {
        return match self.transport.lock() {
            Ok(mut unlocked_transport) => {
                unlocked_transport.write(self.args.return_char.as_bytes())?;

                Ok(())
            }
            Err(err) => Err(ScrapliError {
                details: format!("failed acquiring lock on transport, error: {err}"),
            }),
        };
    }

    /// Write `b` bytes to the device and send a return -- the return character by default is "\n",
    /// but can be configured.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn write_and_return(
        &mut self,
        b: &[u8],
    ) -> Result<(), ScrapliError> {
        self.write(b)?;
        self.write_return()
    }

    /// Return the current "prompt" from the device.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn get_prompt(&mut self) -> Result<Vec<u8>, ScrapliError> {
        self.write_return()?;

        let nb = self.read_until_prompt()?;

        return self.args.prompt_pattern.find(nb.as_slice()).map_or_else(
            || {
                Err(ScrapliError {
                    details: String::from(
                        "read until prompt, but couldn't match prompt, this is a bug",
                    ),
                })
            },
            |b| Ok(b.as_bytes().to_vec()),
        );
    }
}
