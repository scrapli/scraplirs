use super::Channel;
use crate::channel::constants::NEW_LINE_BYTE;
use crate::errors::ScrapliError;
use crate::util::bytes;
use regex::bytes::Regex;
use std::thread;

impl Channel {
    #[allow(clippy::indexing_slicing)]
    fn process_read_buf(
        &self,
        rb: &[u8],
    ) -> Vec<u8> {
        if rb.len() <= self.args.prompt_search_depth.into() {
            return rb.to_vec();
        }

        let mut prb = &rb[(rb.len() - self.args.prompt_search_depth as usize)..];

        let partition_index = prb.iter().position(|&r| r == NEW_LINE_BYTE).unwrap_or(0);

        if partition_index > 0 {
            prb = &prb[partition_index..];
        }

        prb.to_vec()
    }

    /// Reads from the read queue to see if the prompt can be found. This function appends input to
    /// the given read buffer (`rb`) -- it returns a tuple of (bool, result) with the bool
    /// indicating whether or not the prompt has been found.
    pub(super) fn _read_and_check_for_prompt(
        &mut self,
        old_rb: &[u8],
    ) -> (bool, Result<Vec<u8>, ScrapliError>) {
        let mut rb = old_rb.to_vec();

        let nb = match self.read() {
            Ok(nb) => nb,
            Err(err) => return (false, Err(err)),
        };

        if nb.is_empty() {
            return (false, Ok(rb));
        }

        let pnb = self.process_read_buf(nb.as_ref());

        rb.extend(pnb.as_slice());

        if self.args.prompt_pattern.is_match(rb.as_ref()) {
            return (true, Ok(rb));
        }

        (false, Ok(rb))
    }

    /// Read until the `self.args.prompt_pattern` prompt is seen.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn read_until_prompt(&mut self) -> Result<Vec<u8>, ScrapliError> {
        let rb: Vec<u8> = vec![];

        loop {
            let (prompt_found, result) = self._read_and_check_for_prompt(rb.as_slice());

            let rb = match result {
                Ok(rb) => rb,
                Err(err) => return Err(err),
            };

            if prompt_found {
                return Ok(rb);
            }

            thread::sleep(self.args.read_delay);
        }
    }

    /// Reads from the read queue to see if *any* prompt can be found. This function appends input
    /// to the given read buffer (`rb`) -- it returns a tuple of (bool, result) with the bool
    /// indicating whether or not the prompt has been found.
    pub(crate) fn _read_and_check_for_any_prompt(
        &mut self,
        old_rb: &[u8],
        prompts: &[Regex],
    ) -> (bool, Result<Vec<u8>, ScrapliError>) {
        let mut rb = old_rb.to_vec();

        let nb = match self.read() {
            Ok(nb) => nb,
            Err(err) => return (false, Err(err)),
        };

        if nb.is_empty() {
            return (false, Ok(rb));
        }

        let pnb = self.process_read_buf(nb.as_ref());

        rb.extend(pnb.as_slice());

        for prompt in prompts {
            if prompt.is_match(rb.as_ref()) {
                return (true, Ok(rb));
            }
        }

        (false, Ok(rb))
    }

    /// Read until any prompt in the given slice of Regex's is seen.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn read_until_any_prompt(
        &mut self,
        prompts: &[Regex],
    ) -> Result<Vec<u8>, ScrapliError> {
        let rb: Vec<u8> = vec![];

        loop {
            let (prompt_found, result) =
                self._read_and_check_for_any_prompt(rb.as_slice(), prompts);

            let rb = match result {
                Ok(rb) => rb,
                Err(err) => return Err(err),
            };

            if prompt_found {
                return Ok(rb);
            }

            thread::sleep(self.args.read_delay);
        }
    }

    pub(crate) fn _read_and_check_for_fuzzy(
        &mut self,
        old_rb: &[u8],
        explicit: &[u8],
    ) -> (bool, Result<Vec<u8>, ScrapliError>) {
        let mut rb = old_rb.to_vec();

        let nb = match self.read() {
            Ok(nb) => nb,
            Err(err) => return (false, Err(err)),
        };

        if nb.is_empty() {
            return (false, Ok(rb));
        }

        rb.extend(nb.as_slice());

        if bytes::roughly_contains(rb.as_slice(), explicit) {
            return (true, Ok(rb));
        }

        (false, Ok(rb))
    }

    /// Read until an explicit output `b` is seen in the device output, but do so "fuzzily" --
    /// meaning as long as all characters in `b` are seen *in order* in the output we count that as
    /// having "seen" `b`.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn read_until_fuzzy(
        &mut self,
        explicit: &[u8],
    ) -> Result<Vec<u8>, ScrapliError> {
        let mut rb: Vec<u8> = vec![];

        loop {
            let (explicit_found, result) = self._read_and_check_for_fuzzy(rb.as_slice(), explicit);

            rb = match result {
                Ok(rb) => rb,
                Err(err) => return Err(err),
            };

            if explicit_found {
                return Ok(rb);
            }

            thread::sleep(self.args.read_delay);
        }
    }

    pub(crate) fn _read_and_check_for_explicit(
        &mut self,
        old_rb: &[u8],
        explicit: &[u8],
    ) -> (bool, Result<Vec<u8>, ScrapliError>) {
        let mut rb = old_rb.to_vec();

        let nb = match self.read() {
            Ok(nb) => nb,
            Err(err) => return (false, Err(err)),
        };

        if nb.is_empty() {
            return (false, Ok(rb));
        }

        rb.extend(nb.as_slice());

        if bytes::is_sub(rb.as_slice(), explicit) {
            return (true, Ok(rb));
        }

        (false, Ok(rb))
    }

    /// Read until an explicit/exact output `b` is seen in the device output.
    ///
    /// # Errors
    ///
    /// Returns a `ScrapliError` if something that cannot be recovered from occurs.
    pub fn read_until_explicit(
        &mut self,
        explicit: &[u8],
    ) -> Result<Vec<u8>, ScrapliError> {
        let mut rb: Vec<u8> = vec![];

        loop {
            let (explicit_found, result) =
                self._read_and_check_for_explicit(rb.as_slice(), explicit);

            rb = match result {
                Ok(rb) => rb,
                Err(err) => return Err(err),
            };

            if explicit_found {
                return Ok(rb);
            }

            thread::sleep(self.args.read_delay);
        }
    }
}
