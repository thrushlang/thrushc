use crate::core::console::logging;

#[inline]
pub fn print_all_supported_cpus() {
    logging::write(
        logging::OutputIn::Stdout,
        "bleeding-edge
generic
lime1
mvp",
    );
}
