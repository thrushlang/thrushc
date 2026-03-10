/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/


#![allow(clippy::result_unit_err)]

use thrustc_logging::{self, LoggingType};

use inkwell::targets::Target;
use std::path::PathBuf;

#[derive(Debug)]
pub struct JITConfiguration {
    libc_path: PathBuf,
    libraries: Vec<PathBuf>,
    args: Vec<String>,
    entry: Vec<u8>,
}

impl JITConfiguration {
    #[inline]
    pub fn new() -> Self {
        Self {
            libc_path: PathBuf::from(self::get_common_c_runtime_path()),
            libraries: Vec::with_capacity(100),
            args: Vec::with_capacity(100),
            entry: "main".as_bytes().to_vec(),
        }
    }
}

impl JITConfiguration {
    #[inline]
    pub fn get_libraries(&self) -> &[PathBuf] {
        &self.libraries
    }

    #[inline]
    pub fn get_libc_path(&self) -> &PathBuf {
        &self.libc_path
    }

    #[inline]
    pub fn get_entry(&self) -> &[u8] {
        &self.entry
    }

    #[inline]
    pub fn get_args(&self) -> &[String] {
        &self.args
    }
}

impl JITConfiguration {
    #[inline]
    pub fn set_libc_path(&mut self, value: PathBuf) {
        self.libc_path = value;
    }

    #[inline]
    pub fn set_entry(&mut self, value: Vec<u8>) {
        self.entry = value;
    }

    #[inline]
    pub fn add_library(&mut self, value: PathBuf) {
        self.libraries.push(value);
    }

    #[inline]
    pub fn add_argument(&mut self, value: String) {
        self.args.push(value);
    }
}

#[inline]
pub fn has_jit_available(target: &Target) -> Result<(), ()> {
    if !target.has_jit() {
        thrustc_logging::print_error(
            LoggingType::JITCompiler,
            &format!(
                "The JIT compiler isn't properly available for the target '{}'. Aborting compilation.",
                target.get_description().to_string_lossy()
            ),
        );

        return Err(());
    }

    Ok(())
}

fn get_common_c_runtime_path() -> &'static str {
    match std::env::consts::FAMILY {
        "unix" => {
            let candidates: [&str; 5] = [
                "libc.so",
                "libc.so.6",
                "libc.so.1",
                "libc.so.4",
                "libc.so.5",
            ];

            for name in candidates.iter() {
                if unsafe { libloading::Library::new(name) }.is_ok() {
                    return name;
                }
            }

            "libc.so.6"
        }

        "windows" => {
            let candidates: [&str; 7] = [
                "msvcrt.dll",
                "ucrtbase.dll",
                "msvcr120.dll",
                "msvcr110.dll",
                "msvcr100.dll",
                "msvcr90.dll",
                "msvcr80.dll",
            ];

            for name in candidates.iter() {
                if unsafe { libloading::Library::new(name) }.is_ok() {
                    return name;
                }
            }

            "ucrtbase.dll"
        }

        _ => "libc.6.so",
    }
}
