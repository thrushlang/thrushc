use std::path::{Path, PathBuf};

use crate::linkage::checker::signatures::LinkageCheckerSignature;

#[derive(Debug)]
pub struct LinkageCheckerFile<'signature> {
    signatures: Vec<LinkageCheckerSignature<'signature>>,
    path: PathBuf,
}

impl<'signature> LinkageCheckerFile<'signature> {
    #[inline]
    pub fn get_signatures(&self) -> &[LinkageCheckerSignature<'signature>] {
        &self.signatures
    }

    #[inline]
    pub fn get_path(&self) -> &Path {
        &self.path
    }
}
