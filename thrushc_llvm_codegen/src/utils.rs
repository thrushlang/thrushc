pub fn clean_llvm_name(name: &std::ffi::CStr) -> std::borrow::Cow<'_, str> {
    let s: std::borrow::Cow<'_, str> = name.to_string_lossy();

    if let Some(dot_pos) = s.rfind('.') {
        let suffix = &s[dot_pos + 1..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return std::borrow::Cow::Owned(s[..dot_pos].to_string());
        }
    }

    s
}
