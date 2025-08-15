use crate::core::console::logging;

#[inline]
pub fn print_all_supported_cpus() {
    logging::write(
        logging::OutputIn::Stdout,
        "generic
generic-la32
generic-la64
la464
la664
loongarch64",
    );
}
