// SPDX-License-Identifier: GPL-2.0

pub(crate) struct Cr0 {
    pub(crate) value: usize,
}

impl Cr0 {
    pub(crate) const CR0_ENABLE_PAGING: usize = 31;
}

pub(crate) struct RFlags {
    pub(crate) value: u64,
}

#[allow(dead_code)]
impl RFlags {
    pub(crate) const FLAGS_IF: u64 = 9;
    pub(crate) const FLAGS_ZF: u64 = 6;
    pub(crate) const FLAGS_AF: u64 = 4;
    pub(crate) const FLAGS_A1: u64 = 1;
    pub(crate) const FLAGS_CF: u64 = 0;
}

pub(crate) struct Cr4 {
    pub(crate) value: usize,
}

impl Cr4 {
    pub(crate) const CR4_ENABLE_VMX: usize = 13;
}

