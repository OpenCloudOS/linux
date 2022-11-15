// SPDX-License-Identifier: GPL-2.0

#[repr(C)]
#[allow(dead_code)]
pub(crate) struct HostState {
    pub(crate) host_rsp: u64,
    pub(crate) cr3: u64, /* May not match real cr3 */
    pub(crate) cr4: u64, /* May not match real cr4 */
    pub(crate) gs_base: u64,
    pub(crate) fs_base: u64,
    pub(crate) fs_sel: u16,
    pub(crate) gs_sel: u16,
    pub(crate) ldt_sel: u16,
    pub(crate) ds_sel: u16,
    pub(crate) es_sel: u16,
    pub(crate) cr2: u64,
}

impl HostState {
    pub(crate) fn new() -> Self {
        Self {
            host_rsp: 0,
            cr3: 0,
            cr4: 0,
            fs_base: 0,
            gs_base: 0,
            fs_sel: 0,
            gs_sel: 0,
            ldt_sel: 0,
            ds_sel: 0,
            es_sel: 0,
            cr2: 0,
        }
    }
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) struct GuestState {
    //  RIP, RSP, and RFLAGS are automatically saved by VMX in the VMCS.
    pub(crate) host_rsp: u64,
    pub(crate) rax: u64,
    pub(crate) rcx: u64,
    pub(crate) rdx: u64,
    pub(crate) rbx: u64,
    pub(crate) rsp: u64,
    pub(crate) rbp: u64,
    pub(crate) rsi: u64,
    pub(crate) rdi: u64,
    pub(crate) r8: u64,
    pub(crate) r9: u64,
    pub(crate) r10: u64,
    pub(crate) r11: u64,
    pub(crate) r12: u64,
    pub(crate) r13: u64,
    pub(crate) r14: u64,
    pub(crate) r15: u64,
    pub(crate) launched: bool,
    pub(crate) rip: u64,
}

macro_rules! ONE {
    ($x: expr) => {
        (1 + (($x) - ($x)))
    };
}
macro_rules! BITS_SHIFT {
    ($x:expr, $high:expr, $low:expr) => {
        ((($x) >> ($low)) & ((ONE!($x) << (($high) - ($low) + 1)) - 1))
    };
}

#[allow(dead_code)]
impl GuestState {
    pub(crate) fn new() -> Self {
        Self {
            host_rsp: 0,
            rax: 0,
            rcx: 0,
            rdx: 0,
            rbx: 0,
            rsp: 0,
            rbp: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            launched: false,
        }
    }
    // Convenience getters for accessing low 32-bits of common registers.
    pub(crate) fn get_eax(&self) -> u32 {
        return self.rax as u32;
    }
    pub(crate) fn get_ecx(&self) -> u32 {
        return self.rcx as u32;
    }
    pub(crate) fn get_edx(&self) -> u32 {
        return self.rdx as u32;
    }
    pub(crate) fn get_ebx(&self) -> u32 {
        return self.rbx as u32;
    }

    // Convenience getter/setter for fetching the 64-bit value edx:eax, used by
    // several x86_64 instructions, such as `rdmsr` and `wrmsr`.
    //
    // For reads, the top bits of rax and rdx are ignored (c.f. Volume 2C,
    // WRMSR). For writes, the top bits of rax and rdx are set to zero, matching
    // the behaviour of x86_64 instructions such as `rdmsr` (c.f. Volume 2C,
    // RDMSR).

    pub(crate) fn get_edx_eax(&self) -> u64 {
        return (self.get_edx() as u64) << 32 | (self.get_eax() as u64);
    }
    pub(crate) fn set_edx_eax(&mut self, value: u64) {
        self.rax = BITS_SHIFT!(value, 31, 0);
        self.rdx = BITS_SHIFT!(value, 63, 32);
    }
}
