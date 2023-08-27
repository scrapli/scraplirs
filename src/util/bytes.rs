/// Determines if `needle` is a subset of `haystack`.
///
/// # Panics
///
/// It shouldn't panic in context of scraplirs, but... here we are.
#[allow(clippy::expect_used)]
pub fn is_sub<T: PartialEq>(
    mut haystack: &[T],
    needle: &[T],
) -> bool {
    if needle.is_empty() {
        return true;
    }

    while !haystack.is_empty() {
        if haystack.starts_with(needle) {
            return true;
        }

        haystack = haystack.get(1..).expect("slice index out of range");
    }

    false
}

/// Returns true if all characters in `input` show up in order in `output`.
pub fn roughly_contains(
    input: &[u8],
    output: &[u8],
) -> bool {
    if is_sub(input, output) {
        return true;
    }

    if output.len() < input.len() {
        return false;
    }

    let mut iter_output = output;

    for input_char in input {
        let (should_continue, new_output) =
            roughly_contains_iter_output_for_input_char(*input_char, iter_output);

        if should_continue {
            continue;
        }

        iter_output = new_output;

        return false;
    }

    true
}

#[allow(clippy::expect_used)]
fn roughly_contains_iter_output_for_input_char(
    input_char: u8,
    output: &[u8],
) -> (bool, &[u8]) {
    for (idx, output_char) in output.iter().enumerate() {
        if input_char == *output_char {
            return (
                true,
                output.get(idx + 1..).expect("slice index out of range"),
            );
        }
    }

    (false, output)
}

fn char_in_cutset(
    b: u8,
    cutset: &[u8],
) -> bool {
    for cut_b in cutset {
        if b == *cut_b {
            return true;
        }
    }

    false
}

/// Trim all bytes in the cutset from the *left* side of `b`.
///
/// # Panics
///
/// It shouldn't panic in context of scraplirs, but... here we are.
#[allow(dead_code)]
#[allow(clippy::expect_used)]
#[must_use]
pub fn trim_cutset_left<'a>(
    b: &'a [u8],
    cutset: &'a [u8],
) -> &'a [u8] {
    let Some(from) = b.iter().position(|b| !char_in_cutset(*b, cutset)) else {
        return b.get(0..0).expect("slice index out of range");
    };

    b.get(from..).expect("slice index out of range")
}

/// Trim all bytes in the cutset from the *right* side of `b`.
///
/// # Panics
///
/// It shouldn't panic in context of scraplirs, but... here we are.
#[allow(clippy::expect_used)]
#[must_use]
pub fn trim_cutset_right<'a>(
    b: &'a [u8],
    cutset: &'a [u8],
) -> &'a [u8] {
    let to = b
        .iter()
        .rposition(|b| !char_in_cutset(*b, cutset))
        .expect("no char in rposition");

    b.get(..=to).expect("slice index out of range")
}

/// fuck you
///
/// # Panics
///
/// It shouldn't panic in context of scraplirs, but... here we are.
#[allow(clippy::expect_used)]
#[must_use]
pub fn trim_cutset<'a>(
    b: &'a [u8],
    cutset: &'a [u8],
) -> &'a [u8] {
    let Some(from) = b.iter().position(|b| !char_in_cutset(*b, cutset)) else {
        return b.get(0..0).expect("slice index out of range");
    };
    let to = b
        .iter()
        .rposition(|b| !char_in_cutset(*b, cutset))
        .expect("no char in rposition");

    b.get(from..=to).expect("slice index out of range")
}
