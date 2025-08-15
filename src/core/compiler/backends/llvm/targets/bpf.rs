use crate::core::console::logging;

#[inline]
pub fn print_all_supported_cpus() {
    logging::write(
        logging::OutputIn::Stdout,
        "generic
probe
v1
v2
v3
v4",
    );
}
