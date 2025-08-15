use crate::core::console::logging;

#[inline]
pub fn print_all_supported_cpus() {
    logging::write(
        logging::OutputIn::Stdout,
        "arch10
arch11
arch12
arch13
arch14
arch15
arch8
arch9
generic
z10
z13
z14
z15
z16
z17
z196
zEC12",
    );
}
