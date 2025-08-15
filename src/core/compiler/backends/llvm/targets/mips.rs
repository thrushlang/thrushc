use crate::core::console::logging;

#[inline]
pub fn print_all_supported_cpus() {
    logging::write(
        logging::OutputIn::Stdout,
        "generic
mips1
mips2
mips3
mips32
mips32r2
mips32r3
mips32r5
mips32r6
mips4
mips5
mips64
mips64r2
mips64r3
mips64r5
mips64r6
octeon
octeon+
p5600",
    );
}
