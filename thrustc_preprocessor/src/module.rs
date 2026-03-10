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


use uuid::Uuid;

use crate::signatures::Symbol;

#[derive(Debug)]
pub struct Module {
    base_name: String,
    symbols: Vec<Symbol>,
    submodules: Vec<Module>,
    unique_id: Uuid,
}

impl Module {
    pub fn new(base_name: String) -> Self {
        Module {
            base_name,
            symbols: Vec::with_capacity(u8::MAX as usize),
            submodules: Vec::with_capacity(u8::MAX as usize),
            unique_id: Uuid::new_v4(),
        }
    }
}

impl Module {
    #[inline]
    pub fn add_submodule(&mut self, module: Module) {
        self.submodules.push(module);
    }

    #[inline]
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }
}

impl Module {
    #[inline]
    pub fn find_submodule(&self, access: Vec<String>) -> Option<&Module> {
        let mut current_module: &Module = self;

        for name in access.iter() {
            let mut found: bool = false;

            for submodule in &current_module.submodules {
                if submodule.get_name() == name {
                    current_module = submodule;
                    found = true;
                    break;
                }
            }

            if !found {
                return None;
            }
        }

        Some(current_module)
    }
}

impl Module {
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.base_name
    }

    #[inline]
    pub fn get_unique_id(&self) -> &Uuid {
        &self.unique_id
    }
}
