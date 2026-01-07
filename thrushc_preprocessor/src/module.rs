use uuid::Uuid;

use crate::table::ModuleTable;

#[derive(Debug)]
pub struct Module<'module> {
    base_name: String,
    table: ModuleTable<'module>,
    submodules: Vec<Module<'module>>,
    unique_id: Uuid,
}

impl Module<'_> {
    pub fn new(base_name: String) -> Self {
        Module {
            base_name,
            table: ModuleTable::new(),
            submodules: Vec::with_capacity(255),
            unique_id: Uuid::new_v4(),
        }
    }
}

impl<'module> Module<'module> {
    #[inline]
    pub fn add_submodule(&mut self, module: Module<'module>) {
        self.submodules.push(module);
    }
}

impl<'module> Module<'module> {
    #[inline]
    pub fn find_submodule(&self, access: Vec<String>) -> Option<&Module<'module>> {
        let mut current_module: &Module<'_> = self;

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

impl<'module> Module<'module> {
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.base_name
    }

    #[inline]
    pub fn get_table(&self) -> &ModuleTable<'module> {
        &self.table
    }

    #[inline]
    pub fn get_unique_id(&self) -> &Uuid {
        &self.unique_id
    }
}

impl<'module> Module<'module> {
    #[inline]
    pub fn get_mut_table(&mut self) -> &mut ModuleTable<'module> {
        &mut self.table
    }
}
