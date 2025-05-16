use colored::Colorize;

use super::constants::X86_64_SUPPORTED_CPUS;

pub fn is_supported_cpu_target(target: &str) -> bool {
    X86_64_SUPPORTED_CPUS.contains(&target)
}

pub fn print_supported_cpus() {
    println!("Supported CPUs count: {}\n", X86_64_SUPPORTED_CPUS.len());
    println!(
        "Supported {} CPUs:\n",
        "X86_64".custom_color((141, 141, 142)).bold().underline()
    );

    for cpu in X86_64_SUPPORTED_CPUS {
        println!("- {}", cpu);
    }
}
