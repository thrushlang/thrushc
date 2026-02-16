#[derive(Debug, Clone, Copy)]
pub enum DwarfVersion {
    V4,
    V5,
}

#[derive(Debug, Clone, Copy)]
pub struct DebugConfiguration {
    debug_mode: bool,
    debug_for_profiling: bool,
    split_debug_inlining: bool,
    dwarf_version: DwarfVersion,
}

impl DebugConfiguration {
    #[inline]
    pub fn new() -> Self {
        Self {
            debug_mode: false,
            debug_for_profiling: false,
            split_debug_inlining: false,
            dwarf_version: DwarfVersion::V5,
        }
    }
}

impl DebugConfiguration {
    #[inline]
    pub fn set_debug_mode(&mut self) {
        self.debug_mode = true;
    }

    #[inline]
    pub fn set_debug_for_profiling(&mut self) {
        self.debug_for_profiling = true;
    }

    #[inline]
    pub fn set_split_debug_inlining(&mut self) {
        self.split_debug_inlining = true;
    }

    #[inline]
    pub fn set_dwarf_version(&mut self, version: DwarfVersion) {
        self.dwarf_version = version;
    }
}

impl DebugConfiguration {
    #[inline]
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }

    #[inline]
    pub fn need_split_debug_inlining(&self) -> bool {
        self.split_debug_inlining
    }

    #[inline]
    pub fn need_debug_info_for_profiling(&self) -> bool {
        self.debug_for_profiling
    }

    #[inline]
    pub fn get_dwarf_version(&self) -> u64 {
        match self.dwarf_version {
            DwarfVersion::V4 => 4,
            DwarfVersion::V5 => 5,
        }
    }
}
