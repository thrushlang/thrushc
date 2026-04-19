#![allow(non_camel_case_types)]

#[derive(Debug)]
pub struct X86SystemVABI {}

// https://gitlab.com/x86-psABIs/x86-64-ABI - System V
#[derive(Debug, Clone, Copy)]
pub enum X86SystemVABITypeClassification {
    INTEGER,
    SSE,
    SSEUP,
    X87,
    X87UP,
    COMPLEX_X87,
    NO_CLASS,
    MEMORY,
}
