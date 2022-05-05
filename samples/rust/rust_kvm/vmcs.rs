// SPDX-License-Identifier: GPL-2.0
//! Virtual-machine control structure fields.
//!
//! See Intel SDM, Volume 3D, Appendix B.
use kernel::bindings;
use kernel::prelude::*;
use core::arch::global_asm;

// #[feature(global_asm)]

/// VM-execution, VM-exit, and VM-entry control fields

/// Pin-based VM-execution controls.
///
/// A set of bitmask flags useful when setting up [`PINBASED_EXEC_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.6.1.
pub(crate) struct PinbasedControls {
    pub(crate) value: u32,
}

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
pub(crate) struct PrimaryControls {
    pub(crate) value: u32,
}

impl PrimaryControls {
    pub(crate) const HLT_EXITING: u32 = 1 << 7;
    pub(crate) const MWAIT_EXITING: u32 = 1 << 10;
    pub(crate) const MONITOR_EXITING: u32 = 1 << 29;
    pub(crate) const SECONDARY_CONTROLS: u32 = 1 << 31;
}

/// Secondary processor-based VM-execution controls.
///
/// A set of bitmask flags useful when setting up [`SECONDARY_PROCBASED_EXEC_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.6.2, Table 24-7.
pub(crate) struct SecondaryControls {
    pub(crate) value: u32,
}

impl SecondaryControls {
    pub(crate) const UNRESTRICTED_GUEST: u32 = 1 << 7;
    pub(crate) const ENABLE_EPT: u32 = 1 << 1;
}

/// VM-entry controls.
///
/// A set of bitmask flags useful when setting up [`VMENTRY_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.8.
pub(crate) struct EntryControls {
    pub(crate) value: u32,
}

impl EntryControls {
    pub(crate) const LOAD_DEBUG_CONTROLS: u32 = 1 << 2;
}

/// VM-exit controls.
///
/// A set of bitmask flags useful when setting up [`VMEXIT_CONTROLS`] VMCS field.
///
/// See Intel SDM, Volume 3C, Section 24.7.
pub(crate) struct ExitControls {
    pub(crate) value: u32,
}

impl ExitControls {
    pub(crate) const SAVE_DEBUG_CONTROLS: u32 = 1 << 2;
    pub(crate) const HOST_ADDRESS_SPACE_SIZE: u32 = 1 << 9;
    pub(crate) const ACK_INTERRUPT_ON_EXIT: u32 = 1 << 15;
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
    IDT_VECTORING_INFO_FIELD = 0x00004408,
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

pub(crate) fn vmcs_write32(field: VmcsField, value: u32) {
    unsafe {
        bindings::rkvm_vmcs_writel(field as u64, value as u64);
    }
}

pub(crate) fn vmcs_write64(field: VmcsField, value: u64) {
    unsafe {
        bindings::rkvm_vmcs_writel(field as u64, value);
    }
}

pub(crate) fn vmcs_write16(field: VmcsField, value: u16) {
    unsafe {
        bindings::rkvm_vmcs_writel(field as u64, value as u64);
    }
}

pub(crate) fn vmcs_read32(field: VmcsField) -> u32 {
    unsafe {
        let ret = bindings::rkvm_vmcs_readl(field as u64);
        return ret as u32;
    }
}

pub(crate) fn vmcs_read64(field: VmcsField) -> u64 {
    unsafe {
        let ret = bindings::rkvm_vmcs_readl(field as u64);
        return ret;
    }
}

pub(crate) fn vmcs_read16(field: VmcsField) -> u16 {
    unsafe {
        let ret = bindings::rkvm_vmcs_readl(field as u64);
        return ret as u16;
    }
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

    pub(crate) fn setup_config(&mut self) -> u32 {
        let mut v: u64 = 0;
        let mut _pin_based_exec_control: u32 = 0;
        let mut _cpu_based_exec_control: u32 = 0;
        let mut _cpu_based_2nd_exec_control: u32 = 0;
        let mut _vmexit_control: u32 = 0;
        let mut _vmentry_control: u32 = 0;

        _pin_based_exec_control =
            PinbasedControls::EXTERNAL_INTERRUPT_EXITING | PinbasedControls::NMI_EXITING;
        _cpu_based_exec_control = PrimaryControls::HLT_EXITING
            | PrimaryControls::MWAIT_EXITING
            | PrimaryControls::MONITOR_EXITING
            | PrimaryControls::SECONDARY_CONTROLS;
        _cpu_based_2nd_exec_control =
            SecondaryControls::UNRESTRICTED_GUEST | SecondaryControls::ENABLE_EPT;
        _vmexit_control = ExitControls::ACK_INTERRUPT_ON_EXIT
            | ExitControls::HOST_ADDRESS_SPACE_SIZE
            | ExitControls::SAVE_DEBUG_CONTROLS;
        _vmentry_control = EntryControls::LOAD_DEBUG_CONTROLS;

        unsafe {
            v = bindings::rkvm_read_msr(bindings::MSR_IA32_VMX_BASIC);
        }
        let low: u32 = v as u32;
        let high: u32 = (v >> 32) as u32;
        self.size = high & 0x1fff;
        self.basic_cap = high & !0x1fff;
        self.revision_id = low;

        self.pin_based_exec_ctrl = _pin_based_exec_control;
        self.cpu_based_exec_ctrl = _cpu_based_exec_control;
        self.cpu_based_2nd_exec_ctrl = _cpu_based_2nd_exec_control;
        self.vmexit_ctrl = _vmexit_control;
        self.vmentry_ctrl = _vmentry_control;

        return 0;
    }

    pub(crate) fn set_host_constant_vmcs(&self) {
        vmcs_write64(VmcsField::HOST_RIP, vmx_vmexit as u64);
    }

    pub(crate) fn vcpu_vmcs_init(&self) {
        vmcs_write64(VmcsField::VMCS_LINK_POINTER, 0xffffffffffffffff);
        vmcs_write32(
            VmcsField::PIN_BASED_VM_EXEC_CONTROL,
            self.pin_based_exec_ctrl,
        );
        vmcs_write32(
            VmcsField::CPU_BASED_VM_EXEC_CONTROL,
            self.cpu_based_exec_ctrl,
        );
        vmcs_write32(
            VmcsField::SECONDARY_VM_EXEC_CONTROL,
            self.cpu_based_2nd_exec_ctrl,
        );

        vmcs_write32(VmcsField::PAGE_FAULT_ERROR_CODE_MASK, 0);
        vmcs_write32(VmcsField::PAGE_FAULT_ERROR_CODE_MATCH, 0);
        vmcs_write32(VmcsField::CR3_TARGET_COUNT, 0); /* 22.2.1 */

        vmcs_write16(VmcsField::HOST_FS_SELECTOR, 0); /* 22.2.4 */
        vmcs_write16(VmcsField::HOST_GS_SELECTOR, 0); /* 22.2.4 */
        self.set_host_constant_vmcs();
        vmcs_write32(VmcsField::VM_EXIT_CONTROLS, self.vmexit_ctrl);
        vmcs_write32(VmcsField::VM_ENTRY_CONTROLS, self.vmentry_ctrl);
    }
} //impl

extern "C" {
    fn vmx_vmexit();
}

global_asm!(
    "
.global vmx_vmexit
vmx_vmexit:
   ret
"
);
