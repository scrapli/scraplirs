use crate::util::bytes::is_sub;

pub fn string_contains_any_substring(
    s: &str,
    sv: Vec<String>,
) -> bool {
    for ss in sv {
        if is_sub(s.as_bytes(), ss.as_bytes()) {
            return true;
        }
    }

    false
}

pub fn string_vec_contains_substring(
    sv: Vec<String>,
    s: &str,
) -> bool {
    for ss in sv {
        if ss == s {
            return true;
        }
    }

    false
}
