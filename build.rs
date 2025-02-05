use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn main() {
    if cfg!(target_os = "linux") {
        if !Path::new("/usr/include/llvm-c").exists() {
            println!("cargo:warning=LLVM C API not found. Downloading and installing the LLVM Project v17.0.6...");

            let mut wget_command: Command = Command::new("wget");

            wget_command.arg("https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/llvm-project-17.0.6.src.tar.xz");

            let wget_output: Output = wget_command.output().unwrap();

            println!(
                "WGET OUTPUT\n STDOUT: {}\nSTDERR: {}",
                String::from_utf8_lossy(&wget_output.stdout),
                String::from_utf8_lossy(&wget_output.stderr)
            );

            let mut tar_command: Command = Command::new("tar");

            tar_command
                .arg("-xf")
                .arg("llvm-project-17.0.6.src.tar.xz")
                .arg("-C")
                .arg("llvm-project");

            let tar_output: Output = tar_command.output().unwrap();

            println!(
                "TAR OUTPUT\n STDOUT: {}\nSTDERR: {}",
                String::from_utf8_lossy(&tar_output.stdout),
                String::from_utf8_lossy(&tar_output.stderr)
            );

            let mut cmake_command: Command = Command::new("cmake");

            let _ = fs::create_dir_all("llvm-project/llvm/build");

            cmake_command
                .arg("llvm-project/llvm/CMakeLists.txt")
                .arg("-B")
                .arg("llvm-project/llvm/build");

            let cmake_output: Output = cmake_command.output().unwrap();

            println!(
                "CMAKE OUTPUT\n STDOUT: {}\nSTDERR: {}",
                String::from_utf8_lossy(&cmake_output.stdout),
                String::from_utf8_lossy(&cmake_output.stderr)
            );

            let mut make_command: Command = Command::new("make");

            make_command
                .arg("-C")
                .arg("llvm-project/llvm/build/")
                .arg(format!("-j{}", num_cpus::get()))
                .arg("install");

            let make_output: Output = make_command.output().unwrap();

            println!(
                "MAKE OUTPUT\n STDOUT: {}\nSTDERR: {}",
                String::from_utf8_lossy(&make_output.stdout),
                String::from_utf8_lossy(&make_output.stderr)
            );
        }

        return;
    }

    panic!("Build the Compiler is only available on Linux Systems.");
}
