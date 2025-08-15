use crate::core::console::logging;

#[inline]
pub fn print_all_supported_cpus() {
    logging::write(
        logging::OutputIn::Stdout,
        "generic
hexagonv5
hexagonv55
hexagonv60
hexagonv62
hexagonv65
hexagonv66
hexagonv67
hexagonv67t
hexagonv68
hexagonv69
hexagonv71
hexagonv71t
hexagonv73
hexagonv75
hexagonv79",
    );
}
