#[inline]
pub fn set_up() {
    #[cfg(target_os = "windows")]
    {
        colored::control::set_virtual_terminal(true);
        colored::control::set_override(true);
    }

    #[cfg(target_os = "linux")]
    {
        colored::control::set_override(true);
    }
}
