use crate::channel::patterns::ansi_pattern;

/// Strips ansi characters out of the given byte slice.
pub fn strip_ansi(b: &[u8]) -> Vec<u8> {
    ansi_pattern().replace_all(b, vec![]).to_vec()
}
