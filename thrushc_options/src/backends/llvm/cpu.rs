use std::collections::HashSet;

#[derive(Debug)]
pub struct LLVMTargetCPU {
    pub target_cpu: String,
    pub target_cpu_features: String,
}

impl LLVMTargetCPU {
    #[inline]
    pub fn get_cpu_name(&self) -> &str {
        &self.target_cpu
    }

    #[inline]
    pub fn get_cpu_features(&self) -> &str {
        &self.target_cpu_features
    }
}

impl LLVMTargetCPU {
    #[inline]
    pub fn set_cpu_name(&mut self, name: String) {
        self.target_cpu = name;
    }

    #[inline]
    pub fn set_processador_features(&mut self, features: String) {
        self.target_cpu_features = features;
    }

    #[inline]
    pub fn remove_cpu_features(&mut self, blacklist: Vec<&str>) {
        if self.target_cpu_features.is_empty() {
            return;
        }

        let mut kept: Vec<String> = Vec::with_capacity(self.target_cpu_features.len());
        let mut changed: HashSet<&str> = HashSet::with_capacity(blacklist.len());
        let features: Vec<&str> = self.target_cpu_features.split(',').collect();

        for feat in features.iter() {
            let name: &str = &feat[1..];

            if blacklist.contains(&name) {
                kept.push(format!("-{}", name));
                changed.insert(name);
            } else {
                kept.push((*feat).into());
            }
        }

        for supposed_to_be_replaced in blacklist.iter() {
            if !changed.contains(supposed_to_be_replaced) {
                thrushc_logging::print_warn(
                    thrushc_logging::LoggingType::Warning,
                    &format!(
                        "Unable to remove '{}' as a cpu feature, it doesn't exist or match differently.",
                        supposed_to_be_replaced
                    ),
                );
            }
        }

        self.target_cpu_features = kept.join(",");
    }

    #[inline]
    pub fn add_cpu_features(&mut self, blacklist: Vec<&str>) {
        if self.target_cpu_features.is_empty() {
            return;
        }

        let mut kept: Vec<String> = Vec::with_capacity(self.target_cpu_features.len());
        let mut changed: HashSet<&str> = HashSet::with_capacity(blacklist.len());

        let features: Vec<&str> = self.target_cpu_features.split(',').collect();

        for feat in features.iter() {
            let name: &str = &feat[1..];

            if blacklist.contains(&name) {
                kept.push(format!("+{}", name));
                changed.insert(name);
            } else {
                kept.push((*feat).into());
            }
        }

        for supposed_to_be_replaced in blacklist.iter() {
            if !changed.contains(supposed_to_be_replaced) {
                thrushc_logging::print_warn(
                    thrushc_logging::LoggingType::Warning,
                    &format!(
                        "Unable to add '{}' as a cpu feature, it doesn't exist or match differently.",
                        supposed_to_be_replaced
                    ),
                );
            }
        }

        self.target_cpu_features = kept.join(",");
    }
}
