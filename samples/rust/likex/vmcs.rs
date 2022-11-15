// SPDX-License-Identifier: GPL-2.0
//! Virtual-machine control structure fields.
//!
//! See Intel SDM, Volume 3D, Appendix B.
use crate::x86reg::*;
use crate::{rkvm_debug, DEBUG_ON};
use core::arch::asm;
use core::arch::global_asm;
use kernel::bindings;
use kernel::prelude::*;

/// VM-execution, VM-exit, and VM-entry control fields

/// Pin-based VM-execution controls.
///
/// A set of bitmask flags useful when setting up [`PINBASED_EXEC_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.6.1.
#[allow(dead_code)]
pub(crate) struct PinbasedControls {
    pub(crate) value: u32,
}

#[allow(dead_code)]
impl PinbasedControls {
    pub(crate) const EXTERNAL_INTERRUPT_EXITING: u32 = 0;
    pub(crate) const NMI_EXITING: u32 = 3;
    pub(crate) const VIRTUAL_NMIS: u32 = 5;
    pub(crate) const VMX_PREEMPTION_TIMER: u32 = 6;
    pub(crate) const POSTED_INTERRUPTS: u32 = 7;
}
/// Primary processor-based VM-execution controls.
///
/// A set of bitmask flags useful when setting up [`PRIMARY_PROCBASED_EXEC_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.6.2, Table 24-6.
#[allow(dead_code)]
pub(crate) struct PrimaryControls {
    pub(crate) value: u32,
}

#[allow(dead_code)]
impl PrimaryControls {
    pub(crate) const INTERRUPT_WINDOW_EXITING: u32 = 1 << 2;
    pub(crate) const HLT_EXITING: u32 = 1 << 7;
    pub(crate) const MWAIT_EXITING: u32 = 1 << 10;
    pub(crate) const CR3_LOAD_EXITING: u32 = 1 << 15;
    pub(crate) const CR3_STORE_EXITING: u32 = 1 << 16;
    pub(crate) const CR8_LOAD_EXITING: u32 = 1 << 19;
    pub(crate) const CR8_STORE_EXITING: u32 = 1 << 20;
    pub(crate) const MOV_DR_EXITING: u32 = 1 << 23;
    pub(crate) const UNCOND_IO_EXITING: u32 = 1 << 24;
    pub(crate) const USE_IO_BITMAPS: u32 = 1 << 25;
    pub(crate) const MONITOR_EXITING: u32 = 1 << 29;
    pub(crate) const SECONDARY_CONTROLS: u32 = 1 << 31;
}

/// Secondary processor-based VM-execution controls.
///
/// A set of bitmask flags useful when setting up [`SECONDARY_PROCBASED_EXEC_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.6.2, Table 24-7.
#[allow(dead_code)]
pub(crate) struct SecondaryControls {
    pub(crate) value: u32,
}

#[allow(dead_code)]
impl SecondaryControls {
    pub(crate) const UNRESTRICTED_GUEST: u32 = 1 << 7;
    pub(crate) const ENABLE_EPT: u32 = 1 << 1;
    pub(crate) const ENABLE_RDTSCP: u32 = 1 << 3;
    pub(crate) const ENABLE_VPID: u32 = 1 << 5;
    pub(crate) const VIRTUALIZE_X2APIC: u32 = 1 << 4;
    pub(crate) const ENABLE_INVPCID: u32 = 1 << 12;
    pub(crate) const MODE_BASED_EPT: u32 = 1 << 22;
}

/// VM-entry controls.
///
/// A set of bitmask flags useful when setting up [`VMENTRY_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.8.
#[allow(dead_code)]
pub(crate) struct EntryControls {
    pub(crate) value: u32,
}

#[allow(dead_code)]
impl EntryControls {
    pub(crate) const LOAD_DEBUG_CONTROLS: u32 = 1 << 2;
    pub(crate) const IA32E_MODE_GUEST: u32 = 1 << 9;
    pub(crate) const LOAD_IA32_PAT: u32 = 1 << 14;
    pub(crate) const LOAD_IA32_EFER: u32 = 1 << 15;
}

/// VM-exit controls.
///
/// A set of bitmask flags useful when setting up [`VMEXIT_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.7.

#[allow(dead_code)]
pub(crate) struct ExitControls {
    pub(crate) value: u32,
}

#[allow(dead_code)]
impl ExitControls {
    pub(crate) const SAVE_DEBUG_CONTROLS: u32 = 1 << 2;
    pub(crate) const HOST_ADDRESS_SPACE_SIZE: u32 = 1 << 9;
    pub(crate) const ACK_INTERRUPT_ON_EXIT: u32 = 1 << 15;
    pub(crate) const SAVE_IA32_PAT: u32 = 1 << 18;
    pub(crate) const LOAD_IA32_PAT: u32 = 1 << 19;
    pub(crate) const SAVE_IA32_EFER: u32 = 1 << 20;
    pub(crate) const LOAD_IA32_EFER: u32 = 1 << 21;
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub(crate) enum VmcsField {
    VIRTUAL_PROCESSOR_ID = 0x00000000,
    POSTED_INTR_NV = 0x00000002,
    GUEST_ES_SELECTOR = 0x00000800,
    GUEST_CS_SELECTOR = 0x00000802,
    GUEST_SS_SELECTOR = 0x00000804,
    GUEST_DS_SELECTOR = 0x00000806,
    GUEST_FS_SELECTOR = 0x00000808,
    GUEST_GS_SELECTOR = 0x0000080a,
    GUEST_LDTR_SELECTOR = 0x0000080c,
    GUEST_TR_SELECTOR = 0x0000080e,
    GUEST_INTR_STATUS = 0x00000810,
    GUEST_PML_INDEX = 0x00000812,
    HOST_ES_SELECTOR = 0x00000c00,
    HOST_CS_SELECTOR = 0x00000c02,
    HOST_SS_SELECTOR = 0x00000c04,
    HOST_DS_SELECTOR = 0x00000c06,
    HOST_FS_SELECTOR = 0x00000c08,
    HOST_GS_SELECTOR = 0x00000c0a,
    HOST_TR_SELECTOR = 0x00000c0c,
    IO_BITMAP_A = 0x00002000,
    IO_BITMAP_A_HIGH = 0x00002001,
    IO_BITMAP_B = 0x00002002,
    IO_BITMAP_B_HIGH = 0x00002003,
    MSR_BITMAP = 0x00002004,
    MSR_BITMAP_HIGH = 0x00002005,
    VM_EXIT_MSR_STORE_ADDR = 0x00002006,
    VM_EXIT_MSR_STORE_ADDR_HIGH = 0x00002007,
    VM_EXIT_MSR_LOAD_ADDR = 0x00002008,
    VM_EXIT_MSR_LOAD_ADDR_HIGH = 0x00002009,
    VM_ENTRY_MSR_LOAD_ADDR = 0x0000200a,
    VM_ENTRY_MSR_LOAD_ADDR_HIGH = 0x0000200b,
    PML_ADDRESS = 0x0000200e,
    PML_ADDRESS_HIGH = 0x0000200f,
    TSC_OFFSET = 0x00002010,
    TSC_OFFSET_HIGH = 0x00002011,
    VIRTUAL_APIC_PAGE_ADDR = 0x00002012,
    VIRTUAL_APIC_PAGE_ADDR_HIGH = 0x00002013,
    APIC_ACCESS_ADDR = 0x00002014,
    APIC_ACCESS_ADDR_HIGH = 0x00002015,
    POSTED_INTR_DESC_ADDR = 0x00002016,
    POSTED_INTR_DESC_ADDR_HIGH = 0x00002017,
    VM_FUNCTION_CONTROL = 0x00002018,
    VM_FUNCTION_CONTROL_HIGH = 0x00002019,
    EPT_POINTER = 0x0000201a,
    EPT_POINTER_HIGH = 0x0000201b,
    EOI_EXIT_BITMAP0 = 0x0000201c,
    EOI_EXIT_BITMAP0_HIGH = 0x0000201d,
    EOI_EXIT_BITMAP1 = 0x0000201e,
    EOI_EXIT_BITMAP1_HIGH = 0x0000201f,
    EOI_EXIT_BITMAP2 = 0x00002020,
    EOI_EXIT_BITMAP2_HIGH = 0x00002021,
    EOI_EXIT_BITMAP3 = 0x00002022,
    EOI_EXIT_BITMAP3_HIGH = 0x00002023,
    EPTP_LIST_ADDRESS = 0x00002024,
    EPTP_LIST_ADDRESS_HIGH = 0x00002025,
    VMREAD_BITMAP = 0x00002026,
    VMREAD_BITMAP_HIGH = 0x00002027,
    VMWRITE_BITMAP = 0x00002028,
    VMWRITE_BITMAP_HIGH = 0x00002029,
    XSS_EXIT_BITMAP = 0x0000202C,
    XSS_EXIT_BITMAP_HIGH = 0x0000202D,
    ENCLS_EXITING_BITMAP = 0x0000202E,
    ENCLS_EXITING_BITMAP_HIGH = 0x0000202F,
    TSC_MULTIPLIER = 0x00002032,
    TSC_MULTIPLIER_HIGH = 0x00002033,
    GUEST_PHYSICAL_ADDRESS = 0x00002400,
    GUEST_PHYSICAL_ADDRESS_HIGH = 0x00002401,
    VMCS_LINK_POINTER = 0x00002800,
    VMCS_LINK_POINTER_HIGH = 0x00002801,
    GUEST_IA32_DEBUGCTL = 0x00002802,
    GUEST_IA32_DEBUGCTL_HIGH = 0x00002803,
    GUEST_IA32_PAT = 0x00002804,
    GUEST_IA32_PAT_HIGH = 0x00002805,
    GUEST_IA32_EFER = 0x00002806,
    GUEST_IA32_EFER_HIGH = 0x00002807,
    GUEST_IA32_PERF_GLOBAL_CTRL = 0x00002808,
    GUEST_IA32_PERF_GLOBAL_CTRL_HIGH = 0x00002809,
    GUEST_PDPTR0 = 0x0000280a,
    GUEST_PDPTR0_HIGH = 0x0000280b,
    GUEST_PDPTR1 = 0x0000280c,
    GUEST_PDPTR1_HIGH = 0x0000280d,
    GUEST_PDPTR2 = 0x0000280e,
    GUEST_PDPTR2_HIGH = 0x0000280f,
    GUEST_PDPTR3 = 0x00002810,
    GUEST_PDPTR3_HIGH = 0x00002811,
    GUEST_BNDCFGS = 0x00002812,
    GUEST_BNDCFGS_HIGH = 0x00002813,
    GUEST_IA32_RTIT_CTL = 0x00002814,
    GUEST_IA32_RTIT_CTL_HIGH = 0x00002815,
    HOST_IA32_PAT = 0x00002c00,
    HOST_IA32_PAT_HIGH = 0x00002c01,
    HOST_IA32_EFER = 0x00002c02,
    HOST_IA32_EFER_HIGH = 0x00002c03,
    HOST_IA32_PERF_GLOBAL_CTRL = 0x00002c04,
    HOST_IA32_PERF_GLOBAL_CTRL_HIGH = 0x00002c05,
    PIN_BASED_VM_EXEC_CONTROL = 0x00004000,
    CPU_BASED_VM_EXEC_CONTROL = 0x00004002,
    EXCEPTION_BITMAP = 0x00004004,
    PAGE_FAULT_ERROR_CODE_MASK = 0x00004006,
    PAGE_FAULT_ERROR_CODE_MATCH = 0x00004008,
    CR3_TARGET_COUNT = 0x0000400a,
    VM_EXIT_CONTROLS = 0x0000400c,
    VM_EXIT_MSR_STORE_COUNT = 0x0000400e,
    VM_EXIT_MSR_LOAD_COUNT = 0x00004010,
    VM_ENTRY_CONTROLS = 0x00004012,
    VM_ENTRY_MSR_LOAD_COUNT = 0x00004014,
    VM_ENTRY_INTR_INFO_FIELD = 0x00004016,
    VM_ENTRY_EXCEPTION_ERROR_CODE = 0x00004018,
    VM_ENTRY_INSTRUCTION_LEN = 0x0000401a,
    TPR_THRESHOLD = 0x0000401c,
    SECONDARY_VM_EXEC_CONTROL = 0x0000401e,
    PLE_GAP = 0x00004020,
    PLE_WINDOW = 0x00004022,
    VM_INSTRUCTION_ERROR = 0x00004400,
    VM_EXIT_REASON = 0x00004402,
    VM_EXIT_INTR_INFO = 0x00004404,
    VM_EXIT_INTR_ERROR_CODE = 0x00004406,
    IDT_VECTORING_INFO = 0x00004408,
    IDT_VECTORING_ERROR_CODE = 0x0000440a,
    VM_EXIT_INSTRUCTION_LEN = 0x0000440c,
    VMX_INSTRUCTION_INFO = 0x0000440e,
    GUEST_ES_LIMIT = 0x00004800,
    GUEST_CS_LIMIT = 0x00004802,
    GUEST_SS_LIMIT = 0x00004804,
    GUEST_DS_LIMIT = 0x00004806,
    GUEST_FS_LIMIT = 0x00004808,
    GUEST_GS_LIMIT = 0x0000480a,
    GUEST_LDTR_LIMIT = 0x0000480c,
    GUEST_TR_LIMIT = 0x0000480e,
    GUEST_GDTR_LIMIT = 0x00004810,
    GUEST_IDTR_LIMIT = 0x00004812,
    GUEST_ES_AR_BYTES = 0x00004814,
    GUEST_CS_AR_BYTES = 0x00004816,
    GUEST_SS_AR_BYTES = 0x00004818,
    GUEST_DS_AR_BYTES = 0x0000481a,
    GUEST_FS_AR_BYTES = 0x0000481c,
    GUEST_GS_AR_BYTES = 0x0000481e,
    GUEST_LDTR_AR_BYTES = 0x00004820,
    GUEST_TR_AR_BYTES = 0x00004822,
    GUEST_INTERRUPTIBILITY_INFO = 0x00004824,
    GUEST_ACTIVITY_STATE = 0x00004826,
    GUEST_SYSENTER_CS = 0x0000482A,
    VMX_PREEMPTION_TIMER_VALUE = 0x0000482E,
    HOST_IA32_SYSENTER_CS = 0x00004c00,
    CR0_GUEST_HOST_MASK = 0x00006000,
    CR4_GUEST_HOST_MASK = 0x00006002,
    CR0_READ_SHADOW = 0x00006004,
    CR4_READ_SHADOW = 0x00006006,
    CR3_TARGET_VALUE0 = 0x00006008,
    CR3_TARGET_VALUE1 = 0x0000600a,
    CR3_TARGET_VALUE2 = 0x0000600c,
    CR3_TARGET_VALUE3 = 0x0000600e,
    EXIT_QUALIFICATION = 0x00006400,
    GUEST_LINEAR_ADDRESS = 0x0000640a,
    GUEST_CR0 = 0x00006800,
    GUEST_CR3 = 0x00006802,
    GUEST_CR4 = 0x00006804,
    GUEST_ES_BASE = 0x00006806,
    GUEST_CS_BASE = 0x00006808,
    GUEST_SS_BASE = 0x0000680a,
    GUEST_DS_BASE = 0x0000680c,
    GUEST_FS_BASE = 0x0000680e,
    GUEST_GS_BASE = 0x00006810,
    GUEST_LDTR_BASE = 0x00006812,
    GUEST_TR_BASE = 0x00006814,
    GUEST_GDTR_BASE = 0x00006816,
    GUEST_IDTR_BASE = 0x00006818,
    GUEST_DR7 = 0x0000681a,
    GUEST_RSP = 0x0000681c,
    GUEST_RIP = 0x0000681e,
    GUEST_RFLAGS = 0x00006820,
    GUEST_PENDING_DBG_EXCEPTIONS = 0x00006822,
    GUEST_SYSENTER_ESP = 0x00006824,
    GUEST_SYSENTER_EIP = 0x00006826,
    HOST_CR0 = 0x00006c00,
    HOST_CR3 = 0x00006c02,
    HOST_CR4 = 0x00006c04,
    HOST_FS_BASE = 0x00006c06,
    HOST_GS_BASE = 0x00006c08,
    HOST_TR_BASE = 0x00006c0a,
    HOST_GDTR_BASE = 0x00006c0c,
    HOST_IDTR_BASE = 0x00006c0e,
    HOST_IA32_SYSENTER_ESP = 0x00006c10,
    HOST_IA32_SYSENTER_EIP = 0x00006c12,
    HOST_RSP = 0x00006c14,
    HOST_RIP = 0x00006c16,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct VmcsConfig {
    pub(crate) size: u32,
    pub(crate) basic_cap: u32,
    pub(crate) revision_id: u32,
    pub(crate) pin_based_exec_ctrl: u32,
    pub(crate) cpu_based_exec_ctrl: u32,
    pub(crate) cpu_based_2nd_exec_ctrl: u32,
    pub(crate) vmexit_ctrl: u32,
    pub(crate) vmentry_ctrl: u32,
}

fn vmcs_status() -> Result {
    let rflags = read_rflags();
    if rflags & (RFlags::FLAGS_ZF as u64 + RFlags::FLAGS_CF as u64) != 0 {
        return Err(Error::EINVAL);
    }
    Ok(())
}

fn vmcs_write(field: VmcsField, value: u64) -> Result {
    let field = field as u64;
    unsafe {
        asm!("vmwrite {1}, {0};", in(reg) field, in(reg) value, options(att_syntax));
    }
    vmcs_status()
}

pub(crate) fn vmcs_write32(field: VmcsField, value: u32) {
    match vmcs_write(field, value as u64) {
        Ok(()) => return,
        Err(_) => {
            pr_err!(
                " vmcs write error: field={:?}, value = {:x} \n",
                field,
                value
            );
            return;
        }
    }
}

pub(crate) fn vmcs_write64(field: VmcsField, value: u64) {
    match vmcs_write(field, value as u64) {
        Ok(()) => return,
        Err(_) => {
            pr_err!(
                " vmcs write error: field={:?}, value = {:x} \n",
                field,
                value
            );
            return;
        }
    }
}

pub(crate) fn vmcs_write16(field: VmcsField, value: u16) {
    match vmcs_write(field, value as u64) {
        Ok(()) => return,
        Err(_) => {
            pr_err!(
                " vmcs write error: field={:?}, value = {:x} \n",
                field,
                value
            );
            return;
        }
    }
}

fn vmcs_read(field: VmcsField) -> Result<u64> {
    let field = field as u64;
    let mut value: u64 = 0;
    unsafe {
        asm!("vmread {0}, {1};", in(reg) field, out(reg) value, options(att_syntax));
    }
    match vmcs_status() {
        Ok(()) => return Ok(value),
        Err(e) => return Err(e),
    }
}

pub(crate) fn vmcs_read32(field: VmcsField) -> u32 {
    match vmcs_read(field) {
        Ok(val) => return val as u32,
        Err(_) => {
            pr_err!(" vmcs read error: field={:?} \n", field);
            return 0;
        }
    }
}

pub(crate) fn vmcs_read64(field: VmcsField) -> u64 {
    match vmcs_read(field) {
        Ok(val) => return val,
        Err(_) => {
            pr_err!(" vmcs read error: field={:?} \n", field);
            return 0;
        }
    }
}

pub(crate) fn vmcs_read16(field: VmcsField) -> u16 {
    match vmcs_read(field) {
        Ok(val) => return val as u16,
        Err(_) => {
            pr_err!(" vmcs read error: field={:?} \n", field);
            return 0;
        }
    }
}

pub(crate) fn vmclear(addr: u64) -> Result {
    unsafe {
        asm!("vmclear ({0})", in(reg) &addr, options(att_syntax));
    }
    vmcs_status()
}

pub(crate) fn vmptrld(addr: u64) -> Result {
    unsafe {
        asm!("vmptrld ({0})", in(reg) &addr, options(att_syntax));
    }
    vmcs_status()
}

#[repr(u64)]
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum InvEptType {
    /// The logical processor invalidates all mappings associated with bits
    /// 51:12 of the EPT pointer (EPTP) specified in the INVEPT descriptor.
    /// It may invalidate other mappings as well.
    Single = 1,

    /// The logical processor invalidates mappings associated with all EPTPs.
    Global = 2,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct InvEptDesc {
    eptp: u64,
    reserved: u64,
}

pub(crate) fn invept(invalidation: InvEptType, eptp: u64) -> Result {
    let descriptor = InvEptDesc { eptp, reserved: 0 };
    unsafe {
        asm!("invept ({0}), {1}",
       in(reg) &descriptor,
       in(reg) invalidation as u64,
       options(att_syntax));
    }
    vmcs_status()
}

pub(crate) fn read_msr(msr: X86Msr) -> u64 {
    let (high, low): (u32, u32);
    unsafe {
        asm!("rdmsr", out("eax") low, out("edx") high, in("ecx") msr as u32);
    }
    ((high as u64) << 32) | (low as u64)
}

pub(crate) fn write_msr(msr: X86Msr, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        asm!("wrmsr", in("ecx") msr as u32, in("eax") low, in("edx") high);
    }
}

// const X86_EFER_LME: u64 = 0x00000100; /* long mode enable */
// const X86_EFER_LMA: u64 = 0x00000400; /* long mode active */
fn set_control(field: VmcsField, true_msr: u64, old_msr: u64, set: u32, clear: u32) -> Result<u32> {
    let allowed_0 = true_msr as u32;
    let allowed_1 = (true_msr >> 32) as u32;
    if (allowed_1 & set) != set {
        pr_err!("can not set vmcs controls {:?}", field);
        return Err(Error::ENOTSUPP);
    }
    if (!allowed_0 & clear) != clear {
        pr_err!("can not clear vmcs controls {:?}", field);
        return Err(Error::ENOTSUPP);
    }
    if (set & clear) != 0 {
        pr_err!("can not set and clear the same vmcs controls {:?}", field);
        return Err(Error::EINVAL);
    }

    // See Volume 3, Section 31.5.1, Algorithm 3, Part C. If the control can be
    // either 0 or 1 (flexible), and the control is unknown, then refer to the
    // old MSR to find the default value.
    let flexible = allowed_0 ^ allowed_1;
    let unknown = flexible & !(set | clear);
    let defaults = unknown & old_msr as u32;

    Ok(allowed_0 | defaults | set)
}

pub(crate) fn dump_vmcs() {
    pr_info!(" *************************************************************\n");
    pr_info!(" VMCS Host: \n");
    pr_info!(" cr0: {:x} \n", vmcs_read64(VmcsField::HOST_CR0));
    pr_info!(" cr3: {:x} \n", vmcs_read64(VmcsField::HOST_CR3));
    pr_info!(" cr4: {:x} \n", vmcs_read64(VmcsField::HOST_CR4));
    pr_info!(
        " cs-sel: {:x}, ds-sel: {:x} \n",
        vmcs_read16(VmcsField::HOST_CS_SELECTOR),
        vmcs_read16(VmcsField::HOST_DS_SELECTOR)
    );
    pr_info!(
        " es-sel: {:x} , ss-sel: {:x}\n",
        vmcs_read16(VmcsField::HOST_ES_SELECTOR),
        vmcs_read16(VmcsField::HOST_SS_SELECTOR)
    );
    pr_info!(
        " fs-sel: {:x}, gs-sel: {:x} \n",
        vmcs_read16(VmcsField::HOST_FS_SELECTOR),
        vmcs_read16(VmcsField::HOST_GS_SELECTOR)
    );
    pr_info!(" tr-sel: {:x} \n", vmcs_read16(VmcsField::HOST_TR_SELECTOR));
    pr_info!(" pat: {:x} \n", vmcs_read64(VmcsField::HOST_IA32_PAT));
    pr_info!(" efer: {:x} \n", vmcs_read64(VmcsField::HOST_IA32_EFER));
    pr_info!(" fs-base: {:x} \n", vmcs_read64(VmcsField::HOST_FS_BASE));
    pr_info!(" gs-base: {:x} \n", vmcs_read64(VmcsField::HOST_GS_BASE));

    pr_info!(
        " sysenter_esp: {:x} \n",
        vmcs_read64(VmcsField::HOST_IA32_SYSENTER_ESP)
    );
    pr_info!(
        " sysenter_eip: {:x} \n",
        vmcs_read64(VmcsField::HOST_IA32_SYSENTER_EIP)
    );
    pr_info!(
        " sysenter_cs: {:x} \n",
        vmcs_read32(VmcsField::HOST_IA32_SYSENTER_CS)
    );

    pr_info!(" tr_base: {:x} \n", vmcs_read64(VmcsField::HOST_TR_BASE));
    pr_info!(
        " gdtr_base: {:x} \n",
        vmcs_read64(VmcsField::HOST_GDTR_BASE)
    );
    pr_info!(
        " idtr_base: {:x} \n",
        vmcs_read64(VmcsField::HOST_IDTR_BASE)
    );
    pr_info!(
        " rip = {:x}, rsp={:x} \n",
        vmcs_read64(VmcsField::HOST_RIP),
        vmcs_read64(VmcsField::HOST_RSP)
    );
    pr_info!(" *************************************************************\n");
    pr_info!(
        " pin_based={:x} \n",
        vmcs_read32(VmcsField::PIN_BASED_VM_EXEC_CONTROL)
    );
    pr_info!(
        " cpu_based={:x} \n",
        vmcs_read32(VmcsField::CPU_BASED_VM_EXEC_CONTROL)
    );
    pr_info!(
        " cpu2_based={:x} \n",
        vmcs_read32(VmcsField::SECONDARY_VM_EXEC_CONTROL)
    );
    pr_info!(" exit={:x} \n", vmcs_read32(VmcsField::VM_EXIT_CONTROLS));
    pr_info!(" entry={:x} \n", vmcs_read32(VmcsField::VM_ENTRY_CONTROLS));
    pr_info!(" *************************************************************\n");
    pr_info!(" VMCS GUEST \n");
    pr_info!(
        " guest CR0={:x}, CR3={:x},CR4={:x} \n",
        vmcs_read64(VmcsField::GUEST_CR0),
        vmcs_read64(VmcsField::GUEST_CR3),
        vmcs_read64(VmcsField::GUEST_CR4)
    );
    pr_info!(
        " guest cs_base={:x}, ss_base={:x}, es={:x}, fs_base={:x}, gs_base={:x} \n",
        vmcs_read64(VmcsField::GUEST_CS_BASE),
        vmcs_read64(VmcsField::GUEST_SS_BASE),
        vmcs_read64(VmcsField::GUEST_ES_BASE),
        vmcs_read64(VmcsField::GUEST_FS_BASE),
        vmcs_read64(VmcsField::GUEST_GS_BASE)
    );
    pr_info!(
        " guest cs_selector={:x}, rip={:x}, rsp={:x} \n",
        vmcs_read16(VmcsField::GUEST_CS_SELECTOR),
        vmcs_read64(VmcsField::GUEST_RIP),
        vmcs_read64(VmcsField::GUEST_RSP)
    );
    pr_info!(
        " rflags={:x},gdtr={:x}, ldtr={:x}, tr={:x}, idtr={:x} \n",
        vmcs_read64(VmcsField::GUEST_RFLAGS),
        vmcs_read64(VmcsField::GUEST_GDTR_BASE),
        vmcs_read64(VmcsField::GUEST_LDTR_BASE),
        vmcs_read64(VmcsField::GUEST_TR_BASE),
        vmcs_read64(VmcsField::GUEST_IDTR_BASE)
    );
    pr_info!(
        " limit: tr={:x}, ldtr={:x}, cs={:x} \n",
        vmcs_read32(VmcsField::GUEST_TR_LIMIT),
        vmcs_read32(VmcsField::GUEST_LDTR_LIMIT),
        vmcs_read32(VmcsField::GUEST_CS_LIMIT)
    );
    pr_info!(
        " pat= {:x}, efer={:x},activity={:x} \n",
        vmcs_read64(VmcsField::GUEST_IA32_PAT),
        vmcs_read64(VmcsField::GUEST_IA32_EFER),
        vmcs_read32(VmcsField::GUEST_ACTIVITY_STATE)
    );
    pr_info!(
        " interrupt_info={:x} \n",
        vmcs_read32(VmcsField::GUEST_INTERRUPTIBILITY_INFO)
    );
    pr_info!(" *************************************************************\n");
}
impl VmcsConfig {
    pub(crate) fn new() -> Result<Self> {
        let config = Self {
            size: 32,
            basic_cap: 0,
            revision_id: 0,
            pin_based_exec_ctrl: 0,
            cpu_based_exec_ctrl: 0,
            cpu_based_2nd_exec_ctrl: 0,
            vmexit_ctrl: 0,
            vmentry_ctrl: 0,
        };
        Ok(config)
    }

    pub(crate) fn setup_config(&mut self) -> Result<u32> {
        let mut _pin_based_exec_control: u32 = 0;
        let mut _cpu_based_exec_control: u32 = 0;
        let mut _cpu_based_2nd_exec_control: u32 = 0;
        let mut _vmexit_control: u32 = 0;
        let mut _vmentry_control: u32 = 0;

        _pin_based_exec_control =
            PinbasedControls::EXTERNAL_INTERRUPT_EXITING | PinbasedControls::NMI_EXITING;

        _cpu_based_exec_control = PrimaryControls::HLT_EXITING
            | PrimaryControls::MOV_DR_EXITING
            | PrimaryControls::UNCOND_IO_EXITING
            | PrimaryControls::MWAIT_EXITING
            | PrimaryControls::MONITOR_EXITING
            | PrimaryControls::SECONDARY_CONTROLS;
        // | PrimaryControls::CR3_STORE_EXITING
        //| PrimaryControls::CR8_LOAD_EXITING
        // | PrimaryControls::CR8_STORE_EXITING;

        _cpu_based_2nd_exec_control =
            SecondaryControls::UNRESTRICTED_GUEST | SecondaryControls::ENABLE_EPT;
        //| SecondaryControls::ENABLE_RDTSCP
        //| SecondaryControls::ENABLE_VPID
        //| SecondaryControls::ENABLE_INVPCID;

        _vmexit_control = ExitControls::HOST_ADDRESS_SPACE_SIZE
              //| ExitControls::SAVE_DEBUG_CONTROLS
              | ExitControls::ACK_INTERRUPT_ON_EXIT
              | ExitControls::SAVE_IA32_PAT
              | ExitControls::LOAD_IA32_PAT
              | ExitControls::SAVE_IA32_EFER
              | ExitControls::LOAD_IA32_EFER;

        _vmentry_control = //EntryControls::IA32E_MODE_GUEST
                          EntryControls::LOAD_IA32_PAT
                           | EntryControls::LOAD_IA32_EFER;

        let v = read_msr(X86Msr::VMX_BASIC);
        let low: u32 = v as u32;
        let high: u32 = (v >> 32) as u32;
        if ((high >> 18) & 15) != 6 {
            rkvm_debug!(" vmcs access mem type is not WB, high={:x} \n", high);
        }
        self.size = high & 0x1fff;
        self.basic_cap = high & !0x1fff;
        self.revision_id = low;

        let truev = read_msr(X86Msr::TRUE_PINBASED_CTLS);
        let old = read_msr(X86Msr::PINBASED_CTLS);
        self.pin_based_exec_ctrl = set_control(
            VmcsField::PIN_BASED_VM_EXEC_CONTROL,
            truev,
            old,
            _pin_based_exec_control,
            0,
        )?;
        let truev = read_msr(X86Msr::TRUE_PROCBASED_CTLS);
        let old = read_msr(X86Msr::PROCBASED_CTLS);
        self.cpu_based_exec_ctrl = set_control(
            VmcsField::CPU_BASED_VM_EXEC_CONTROL,
            truev,
            old,
            _cpu_based_exec_control,
            0,
        )?;
        let truev = read_msr(X86Msr::PROCBASED_CTLS2);
        self.cpu_based_2nd_exec_ctrl = set_control(
            VmcsField::SECONDARY_VM_EXEC_CONTROL,
            truev,
            0,
            _cpu_based_2nd_exec_control,
            0,
        )?;
        let truev = read_msr(X86Msr::TRUE_EXIT_CTLS);
        let old = read_msr(X86Msr::EXIT_CTLS);
        self.vmexit_ctrl =
            set_control(VmcsField::VM_EXIT_CONTROLS, truev, old, _vmexit_control, 0)?;
        let truev = read_msr(X86Msr::TRUE_ENTRY_CTLS);
        let old = read_msr(X86Msr::ENTRY_CTLS);
        self.vmentry_ctrl = set_control(
            VmcsField::VM_ENTRY_CONTROLS,
            truev,
            old,
            _vmentry_control,
            0,
        )?;

        rkvm_debug!(
            " setup pin={:x},cpu={:x}, cpu2={:x},exit={:x}, entry={:x} \n",
            self.pin_based_exec_ctrl,
            self.cpu_based_exec_ctrl,
            self.cpu_based_2nd_exec_ctrl,
            self.vmexit_ctrl,
            self.vmentry_ctrl
        );
        Ok(0)
    }
    pub(crate) fn set_host_constant_vmcs(&self) {
        let mut cr0 = read_cr0();
        cr0 &= !(Cr0::CR0_TS as u64);
        let cr3 = read_cr3();
        // let cr4 = unsafe { bindings::cr4_read_shadow() };
        vmcs_write64(VmcsField::HOST_CR0, cr0);
        vmcs_write64(VmcsField::HOST_CR3, cr3);
        vmcs_write64(VmcsField::HOST_CR4, 0x7726e0);
        vmcs_write16(VmcsField::HOST_CS_SELECTOR, 16);
        vmcs_write16(VmcsField::HOST_DS_SELECTOR, 0);
        vmcs_write16(VmcsField::HOST_ES_SELECTOR, 0);
        vmcs_write16(VmcsField::HOST_SS_SELECTOR, 24);

        vmcs_write16(VmcsField::HOST_FS_SELECTOR, 0); /* 22.2.4 */
        vmcs_write16(VmcsField::HOST_GS_SELECTOR, 0); /* 22.2.4 */
        vmcs_write16(VmcsField::HOST_TR_SELECTOR, 64);
        // let fs = read_msr(X86Msr::FS_BASE);
        let fs = read_fsbase();
        vmcs_write64(VmcsField::HOST_FS_BASE, fs);
        // let gs = read_msr(X86Msr::GS_BASE);
        let gs = unsafe { bindings::rkvm_rdgsbase() };
        vmcs_write64(VmcsField::HOST_GS_BASE, gs);

        // from kvm
        vmcs_write64(VmcsField::HOST_IA32_SYSENTER_ESP, 0);
        vmcs_write64(VmcsField::HOST_IA32_SYSENTER_EIP, 0);
        vmcs_write32(VmcsField::HOST_IA32_SYSENTER_CS, 0);

        let gdt = unsafe { bindings::rkvm_get_current_gdt_ro() };
        let tss = unsafe { bindings::rkvm_get_current_tss_ro() };
        rkvm_debug!(
            "get gdt={:x}, tss={:x}, fs={:x}, gs={:x}  \n",
            gdt,
            tss,
            fs,
            gs
        );

        vmcs_write64(VmcsField::HOST_TR_BASE, tss);
        vmcs_write64(VmcsField::HOST_GDTR_BASE, gdt);
        vmcs_write64(VmcsField::HOST_IDTR_BASE, 0xfffffe0000000000);
        vmcs_write32(VmcsField::VM_EXIT_MSR_LOAD_COUNT, 0);
        vmcs_write64(VmcsField::HOST_RIP, vmx_exit as u64);
    }

    pub(crate) fn vcpu_vmcs_init(&mut self) {
        self.setup_config();
        vmcs_write64(VmcsField::VMCS_LINK_POINTER, 0xffffffffffffffff);
        vmcs_write32(
            VmcsField::PIN_BASED_VM_EXEC_CONTROL,
            self.pin_based_exec_ctrl,
        );
        rkvm_debug!("  pin_based = {:x} \n", self.pin_based_exec_ctrl);

        vmcs_write32(
            VmcsField::CPU_BASED_VM_EXEC_CONTROL,
            self.cpu_based_exec_ctrl,
        );
        rkvm_debug!("  cpu_based = {:x} \n", self.cpu_based_exec_ctrl);

        vmcs_write32(
            VmcsField::SECONDARY_VM_EXEC_CONTROL,
            self.cpu_based_2nd_exec_ctrl,
        );
        rkvm_debug!("  cpu_2nd_based = {:x} \n", self.cpu_based_2nd_exec_ctrl);

        vmcs_write32(VmcsField::EXCEPTION_BITMAP, 0x60042);
        vmcs_write32(VmcsField::PAGE_FAULT_ERROR_CODE_MASK, 0);
        vmcs_write32(VmcsField::PAGE_FAULT_ERROR_CODE_MATCH, 0);
        vmcs_write32(VmcsField::CR3_TARGET_COUNT, 0); /* 22.2.1 */
        self.set_host_constant_vmcs();
        vmcs_write32(VmcsField::VM_EXIT_CONTROLS, self.vmexit_ctrl);
        vmcs_write32(VmcsField::VM_ENTRY_CONTROLS, self.vmentry_ctrl);

        rkvm_debug!("  vmexit_ctrl = {:x} \n", self.vmexit_ctrl);
        rkvm_debug!("  vmentry_ctrl = {:x} \n", self.vmentry_ctrl);

        vmcs_write32(VmcsField::VM_ENTRY_MSR_LOAD_COUNT, 0);
        vmcs_write32(VmcsField::VM_ENTRY_INTR_INFO_FIELD, 0);
        vmcs_write32(VmcsField::VM_EXIT_MSR_LOAD_COUNT, 0);
        vmcs_write32(VmcsField::VM_EXIT_MSR_STORE_COUNT, 0);

        vmcs_write64(VmcsField::GUEST_CR0, 0x30);
        vmcs_write64(VmcsField::CR0_READ_SHADOW, 0x60000010);
        vmcs_write64(VmcsField::CR0_GUEST_HOST_MASK, 0xfffffffffffffff7);

        vmcs_write64(VmcsField::GUEST_CR4, 0x00002040);
        vmcs_write64(VmcsField::CR4_GUEST_HOST_MASK, 0xfffffffffffef871);
        vmcs_write64(VmcsField::CR4_READ_SHADOW, 0);

        vmcs_write32(VmcsField::GUEST_SYSENTER_CS, 0);
        vmcs_write64(VmcsField::GUEST_SYSENTER_ESP, 0);
        vmcs_write64(VmcsField::GUEST_SYSENTER_EIP, 0);
        vmcs_write64(VmcsField::GUEST_IA32_DEBUGCTL, 0);
        // vcpu reset
        vmcs_write16(VmcsField::GUEST_CS_SELECTOR, 0x0);
        vmcs_write64(VmcsField::GUEST_CS_BASE, 0x0);
        vmcs_write32(VmcsField::GUEST_CS_LIMIT, 0xffff);
        vmcs_write32(VmcsField::GUEST_CS_AR_BYTES, 0x009b);

        vmcs_write16(VmcsField::GUEST_TR_SELECTOR, 0);
        vmcs_write64(VmcsField::GUEST_TR_BASE, 0);
        vmcs_write32(VmcsField::GUEST_TR_LIMIT, 0xffff);
        vmcs_write32(VmcsField::GUEST_TR_AR_BYTES, 0x008b);

        vmcs_write64(VmcsField::GUEST_DS_BASE, 0);
        vmcs_write32(VmcsField::GUEST_DS_LIMIT, 0xffff);
        vmcs_write16(VmcsField::GUEST_DS_SELECTOR, 0x0);
        vmcs_write32(VmcsField::GUEST_DS_AR_BYTES, 0x0093);

        vmcs_write64(VmcsField::GUEST_SS_BASE, 0);
        vmcs_write32(VmcsField::GUEST_SS_LIMIT, 0xffff);
        vmcs_write16(VmcsField::GUEST_SS_SELECTOR, 0x0);
        vmcs_write32(VmcsField::GUEST_SS_AR_BYTES, 0x0093);

        vmcs_write64(VmcsField::GUEST_ES_BASE, 0);
        vmcs_write32(VmcsField::GUEST_ES_LIMIT, 0xffff);
        vmcs_write16(VmcsField::GUEST_ES_SELECTOR, 0x0);
        vmcs_write32(VmcsField::GUEST_ES_AR_BYTES, 0x0093);

        vmcs_write64(VmcsField::GUEST_FS_BASE, 0);
        vmcs_write32(VmcsField::GUEST_FS_LIMIT, 0xffff);
        vmcs_write16(VmcsField::GUEST_FS_SELECTOR, 0x0);
        vmcs_write32(VmcsField::GUEST_FS_AR_BYTES, 0x0093);

        vmcs_write64(VmcsField::GUEST_GS_BASE, 0);
        vmcs_write32(VmcsField::GUEST_GS_LIMIT, 0xffff);
        vmcs_write16(VmcsField::GUEST_GS_SELECTOR, 0x0);
        vmcs_write32(VmcsField::GUEST_GS_AR_BYTES, 0x0093);

        vmcs_write16(VmcsField::GUEST_LDTR_SELECTOR, 0);
        vmcs_write64(VmcsField::GUEST_LDTR_BASE, 0);
        vmcs_write32(VmcsField::GUEST_LDTR_LIMIT, 0xffff);
        vmcs_write32(VmcsField::GUEST_LDTR_AR_BYTES, 0x00082);

        vmcs_write64(VmcsField::GUEST_GDTR_BASE, 0);
        vmcs_write32(VmcsField::GUEST_GDTR_LIMIT, 0xffff);
        vmcs_write64(VmcsField::GUEST_IDTR_BASE, 0);
        vmcs_write32(VmcsField::GUEST_IDTR_LIMIT, 0xffff);

        vmcs_write64(VmcsField::GUEST_RFLAGS, 0x0);

        vmcs_write32(VmcsField::GUEST_ACTIVITY_STATE, 0);
        vmcs_write32(VmcsField::GUEST_INTERRUPTIBILITY_INFO, 0);

        vmcs_write64(VmcsField::GUEST_RSP, 0);
        vmcs_write64(VmcsField::GUEST_CR3, 0);

        let pat = read_msr(X86Msr::PAT);
        vmcs_write64(VmcsField::GUEST_IA32_PAT, 0x7040600070406);
        vmcs_write64(VmcsField::HOST_IA32_PAT, pat);
        let efer = read_msr(X86Msr::EFER);
        vmcs_write64(VmcsField::HOST_IA32_EFER, efer);

        rkvm_debug!(" host_efer = {:x} \n", efer);
        // efer &= !(X86_EFER_LME | X86_EFER_LMA);
        vmcs_write64(VmcsField::GUEST_IA32_EFER, /*efer*/ 0);
        vmcs_write32(VmcsField::CR3_TARGET_COUNT, 0);
        vmcs_write64(VmcsField::TSC_OFFSET, 0);
        vmcs_write64(VmcsField::GUEST_DR7, 0x400);
        vmcs_write64(VmcsField::GUEST_IA32_DEBUGCTL, 0);
        vmcs_write64(VmcsField::GUEST_PENDING_DBG_EXCEPTIONS, 0);

        //vmcs_write16(VmcsField::VIRTUAL_PROCESSOR_ID, 1);
        rkvm_debug!(" vcpu_vmcs_init \n");
    }
} //impl

extern "C" {
    fn vmx_exit() -> u32;
}

global_asm!(
    "
.global vmx_exit
vmx_exit:
    // Store the guest registers not covered by the VMCS. At this point,
    // guest_state is in RSP.
    add     rsp, 17 * 8
    push    r15
    push    r14
    push    r13
    push    r12
    push    r11
    push    r10
    push    r9
    push    r8
    push    rdi
    push    rsi
    push    rbp 
    sub     rsp, 8 //rsp
    push    rbx
    push    rdx
    push    rcx
    push    rax
    sub     rsp, 8

    pop     rsp

   // Load host callee save registers, return address, and processor flags.
    pop     rbx
    pop     r12
    pop     r13
    pop     r14
    pop     r15
    pop     rbp

    // Return false
    xor     rax, rax
    ret
"
);
