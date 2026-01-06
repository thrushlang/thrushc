use uuid::Uuid;

use crate::table::ModuleTable;

#[derive(Debug)]
pub struct Module<'module> {
    name: String,
    table: ModuleTable<'module>,
    unique_id: Uuid,
}

impl Module<'_> {
    pub fn new(name: String) -> Self {
        Module {
            name,
            table: ModuleTable::new(),
            unique_id: Uuid::new_v4(),
        }
    }
}

impl<'module> Module<'module> {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_table(&self) -> &ModuleTable<'module> {
        &self.table
    }

    pub fn get_unique_id(&self) -> &Uuid {
        &self.unique_id
    }
}

impl<'module> Module<'module> {
    pub fn get_mut_table(&mut self) -> &mut ModuleTable<'module> {
        &mut self.table
    }
}
