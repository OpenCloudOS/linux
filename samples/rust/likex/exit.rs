// SPDX-License-Identifier: GPL-2.0
use super::{VcpuWrapper, RkvmRun};
use crate::mmu::*;
use crate::vmcs::*;
use crate::{rkvm_debug, DEBUG_ON};
use core::arch::asm;
use core::mem::MaybeUninit;
use kernel::prelude::*;
use kernel::{bindings, bit, sync::Ref, Error, Result, PAGE_SIZE};

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub(crate) enum RkvmUserExitReason {
    RKVM_EXIT_UNKNOWN = 0,
    RKVM_EXIT_EXCEPTION = 1,
    RKVM_EXIT_IO = 2,
    RKVM_EXIT_HYPERCALL = 3,
    RKVM_EXIT_DEBUG = 4,
    RKVM_EXIT_HLT = 5,
    RKVM_EXIT_MMIO = 6,
    RKVM_EXIT_IRQ_WINDOW_OPEN = 7,
    RKVM_EXIT_SHUTDOWN = 8,
    RKVM_EXIT_FAIL_ENTRY = 9,
    RKVM_EXIT_INTR = 10,
    RKVM_EXIT_SET_TPR = 11,
    RKVM_EXIT_TPR_ACCESS = 12,
    RKVM_EXIT_S390_SIEIC = 13,
    RKVM_EXIT_S390_RESET = 14,
    RKVM_EXIT_DCR = 15,
    RKVM_EXIT_NMI = 16,
    RKVM_EXIT_INTERNAL_ERROR = 17,
    RKVM_EXIT_OSI = 18,
    RKVM_EXIT_PAPR_HCALL = 19,
    RKVM_EXIT_S390_UCONTROL = 20,
    RKVM_EXIT_WATCHDOG = 21,
    RKVM_EXIT_S390_TSCH = 22,
    RKVM_EXIT_EPR = 23,
    RKVM_EXIT_SYSTEM_EVENT = 24,
    RKVM_EXIT_S390_STSI = 25,
    RKVM_EXIT_IOAPIC_EOI = 26,
    RKVM_EXIT_HYPERV = 27,
    RKVM_EXIT_ARM_NISV = 28,
    RKVM_EXIT_X86_RDMSR = 29,
    RKVM_EXIT_X86_WRMSR = 30,
    RKVM_EXIT_DIRTY_RING_FULL = 31,
    RKVM_EXIT_AP_RESET_HOLD = 32,
    RKVM_EXIT_X86_BUS_LOCK = 33,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub(crate) enum ExitReason {
    EXCEPTION_OR_NMI = 0,
    EXTERNAL_INTERRUPT = 1,
    TRIPLE_FAULT = 2,
    INIT_SIGNAL = 3,
    STARTUP_IPI = 4,
    IO_SMI = 5,
    OTHER_SMI = 6,
    INTERRUPT_WINDOW = 7,
    NMI_WINDOW = 8,
    TASK_SWITCH = 9,
    CPUID = 10,
    GETSEC = 11,
    HLT = 12,
    INVD = 13,
    INVLPG = 14,
    RDPMC = 15,
    RDTSC = 16,
    RSM = 17,
    VMCALL = 18,
    VMCLEAR = 19,
    VMLAUNCH = 20,
    VMPTRLD = 21,
    VMPTRST = 22,
    VMREAD = 23,
    VMRESUME = 24,
    VMWRITE = 25,
    VMXOFF = 26,
    VMXON = 27,
    CONTROL_REGISTER_ACCESS = 28,
    MOV_DR = 29,
    IO_INSTRUCTION = 30,
    RDMSR = 31,
    WRMSR = 32,
    ENTRY_FAILURE_GUEST_STATE = 33,
    ENTRY_FAILURE_MSR_LOADING = 34,
    MWAIT = 36,
    MONITOR_TRAP_FLAG = 37,
    MONITOR = 39,
    PAUSE = 40,
    ENTRY_FAILURE_MACHINE_CHECK = 41,
    TPR_BELOW_THRESHOLD = 43,
    APIC_ACCESS = 44,
    VIRTUALIZED_EOI = 45,
    ACCESS_GDTR_OR_IDTR = 46,
    ACCESS_LDTR_OR_TR = 47,
    EPT_VIOLATION = 48,
    EPT_MISCONFIGURATION = 49,
    INVEPT = 50,
    RDTSCP = 51,
    VMX_PREEMPT_TIMER_EXPIRED = 52,
    INVVPID = 53,
    WBINVD = 54,
    XSETBV = 55,
    APIC_WRITE = 56,
    RDRAND = 57,
    INVPCID = 58,
    VMFUNC = 59,
    ENCLS = 60,
    RDSEED = 61,
    PAGE_MODIFICATION_LOG_FULL = 62,
    XSAVES = 63,
    XRSTORS = 64,
    SPP_EVENT = 66,
    UMWAIT = 67,
    TPAUSE = 68,
    UNKNOWN = 200,
}

impl From<ExitReason> for RkvmUserExitReason {
    fn from(er: ExitReason) -> Self {
        match er {
            ExitReason::UNKNOWN => RkvmUserExitReason::RKVM_EXIT_UNKNOWN,
            ExitReason::HLT => RkvmUserExitReason::RKVM_EXIT_HLT,
            ExitReason::IO_INSTRUCTION => RkvmUserExitReason::RKVM_EXIT_IO,
            _ => RkvmUserExitReason::RKVM_EXIT_UNKNOWN,
        }
    }
}

impl From<u32> for RkvmUserExitReason {
    fn from(v: u32) -> Self {
        match v {
            0 => RkvmUserExitReason::RKVM_EXIT_UNKNOWN,
            1 => RkvmUserExitReason::RKVM_EXIT_EXCEPTION,
            2 => RkvmUserExitReason::RKVM_EXIT_IO,
            3 => RkvmUserExitReason::RKVM_EXIT_HYPERCALL,
            4 => RkvmUserExitReason::RKVM_EXIT_DEBUG,
            5 => RkvmUserExitReason::RKVM_EXIT_HLT,
            6 => RkvmUserExitReason::RKVM_EXIT_MMIO,
            7 => RkvmUserExitReason::RKVM_EXIT_IRQ_WINDOW_OPEN,
            8 => RkvmUserExitReason::RKVM_EXIT_SHUTDOWN,
            9 => RkvmUserExitReason::RKVM_EXIT_FAIL_ENTRY,
            10 => RkvmUserExitReason::RKVM_EXIT_INTR,
            11 => RkvmUserExitReason::RKVM_EXIT_SET_TPR,
            12 => RkvmUserExitReason::RKVM_EXIT_TPR_ACCESS,
            15 => RkvmUserExitReason::RKVM_EXIT_DCR,
            16 => RkvmUserExitReason::RKVM_EXIT_NMI,
            17 => RkvmUserExitReason::RKVM_EXIT_INTERNAL_ERROR,
            18 => RkvmUserExitReason::RKVM_EXIT_OSI,
            19 => RkvmUserExitReason::RKVM_EXIT_PAPR_HCALL,
            21 => RkvmUserExitReason::RKVM_EXIT_WATCHDOG,
            23 => RkvmUserExitReason::RKVM_EXIT_EPR,
            24 => RkvmUserExitReason::RKVM_EXIT_SYSTEM_EVENT,
            _ => RkvmUserExitReason::RKVM_EXIT_UNKNOWN,
        }
    }
}

impl From<u32> for ExitReason {
    fn from(v: u32) -> Self {
        match v {
            0 => ExitReason::EXCEPTION_OR_NMI,
            1 => ExitReason::EXTERNAL_INTERRUPT,
            2 => ExitReason::TRIPLE_FAULT,
            3 => ExitReason::INIT_SIGNAL,
            4 => ExitReason::STARTUP_IPI,
            5 => ExitReason::IO_SMI,
            6 => ExitReason::OTHER_SMI,
            7 => ExitReason::INTERRUPT_WINDOW,
            8 => ExitReason::NMI_WINDOW,
            9 => ExitReason::TASK_SWITCH,
            10 => ExitReason::CPUID,
            11 => ExitReason::GETSEC,
            12 => ExitReason::HLT,
            13 => ExitReason::INVD,
            14 => ExitReason::INVLPG,
            15 => ExitReason::RDPMC,
            16 => ExitReason::RDTSC,
            17 => ExitReason::RSM,
            18 => ExitReason::VMCALL,
            28 => ExitReason::CONTROL_REGISTER_ACCESS,
            29 => ExitReason::MOV_DR,
            30 => ExitReason::IO_INSTRUCTION,
            31 => ExitReason::RDMSR,
            32 => ExitReason::WRMSR,
            33 => ExitReason::ENTRY_FAILURE_GUEST_STATE,
            34 => ExitReason::ENTRY_FAILURE_MSR_LOADING,
            36 => ExitReason::MWAIT,
            37 => ExitReason::MONITOR_TRAP_FLAG,
            39 => ExitReason::MONITOR,
            40 => ExitReason::PAUSE,
            41 => ExitReason::ENTRY_FAILURE_MACHINE_CHECK,
            43 => ExitReason::TPR_BELOW_THRESHOLD,
            44 => ExitReason::APIC_ACCESS,
            45 => ExitReason::VIRTUALIZED_EOI,
            46 => ExitReason::ACCESS_GDTR_OR_IDTR,
            47 => ExitReason::ACCESS_LDTR_OR_TR,
            48 => ExitReason::EPT_VIOLATION,
            49 => ExitReason::EPT_MISCONFIGURATION,
            50 => ExitReason::INVEPT,
            51 => ExitReason::RDTSCP,
            52 => ExitReason::VMX_PREEMPT_TIMER_EXPIRED,
            53 => ExitReason::INVVPID,
            54 => ExitReason::WBINVD,
            55 => ExitReason::XSETBV,
            56 => ExitReason::APIC_WRITE,
            57 => ExitReason::RDRAND,
            58 => ExitReason::INVPCID,
            59 => ExitReason::VMFUNC,
            60 => ExitReason::ENCLS,
            61 => ExitReason::RDSEED,
            62 => ExitReason::PAGE_MODIFICATION_LOG_FULL,
            _ => ExitReason::UNKNOWN,
        }
    }
}

#[repr(u64)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum PferrMask {
    PFERR_PRESENT_MASK = 1,
    PFERR_WRITE_MASK = 2,
    PFERR_USER_MASK = 4,
    PFERR_RSVD_MASK = 8,
    PFERR_FETCH_MASK = 16,
    PFERR_GUEST_FINAL_MASK = 1 << 32,
    PFERR_GUEST_PAGE_MASK = 1 << 33,
}

#[repr(u64)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum EptViolationMask {
    EPT_VIOLATION_ACC_READ = 1,
    EPT_VIOLATION_ACC_WRITE = 2,
    EPT_VIOLATION_ACC_INSTR = 4,
    EPT_VIOLATION_READABLE = 8,
    EPT_VIOLATION_WRITABLE = 16,
    EPT_VIOLATION_EXECUTABLE = 32,
    EPT_VIOLATION_GVA_TRANSLATED = 64,
}

#[derive(Debug)]
struct RkvmPageFault {
    addr: u64,
    error_code: u64,
    prefetch: bool,
    //get from error_code
    //exec: bool,
    //write: bool,
    present: bool,
    //rsvd: bool,
    //user: bool,
    gfn: u64,
    //output
    pfn: u64,
    hva: u64,
    goal_level: u64,
}

pub(crate) struct ExitInfo {
    pub(crate) entry_failure: bool,
    pub(crate) exit_reason: ExitReason,
    pub(crate) exit_instruction_length: u32,
    pub(crate) exit_qualification: u64,
    pub(crate) guest_rip: u64,
}

impl ExitInfo {
    pub(crate) fn from_vmcs() -> Self {
        let full_reason = vmcs_read32(VmcsField::VM_EXIT_REASON);
        Self {
            exit_reason: (full_reason & 0x1ffffu32).try_into().unwrap(),
            entry_failure: (full_reason & bit(31)) != 0,
            exit_instruction_length: vmcs_read32(VmcsField::VM_EXIT_INSTRUCTION_LEN),
            exit_qualification: vmcs_read64(VmcsField::EXIT_QUALIFICATION),
            guest_rip: vmcs_read64(VmcsField::GUEST_RIP),
        }
    }

    fn next_rip(&self) {
        vmcs_write64(
            VmcsField::GUEST_RIP,
            self.guest_rip + self.exit_instruction_length as u64,
        );
    }
}

pub(crate) fn handle_hlt(exit_info: &ExitInfo, vcpu: &VcpuWrapper) -> Result<u64> {
    let vcpuinner = vcpu.vcpuinner.lock();
    unsafe {
        (*(vcpuinner.run.as_mut_ptr::<RkvmRun>())).exit_reason =
            (RkvmUserExitReason::from(exit_info.exit_reason)) as u32;
    }
    exit_info.next_rip();
    Ok(0)
}

pub(crate) fn handle_io(exit_info: &ExitInfo, vcpu: &VcpuWrapper) -> Result<u64> {
    let exit_qualification = exit_info.exit_qualification;
    let vcpuinner = vcpu.vcpuinner.lock();
    unsafe {
        (*(vcpuinner.run.as_mut_ptr::<RkvmRun>())).io.port = (exit_qualification >> 16) as u16;
        (*(vcpuinner.run.as_mut_ptr::<RkvmRun>())).io.size = ((exit_qualification & 7) + 1) as u8;
        (*(vcpuinner.run.as_mut_ptr::<RkvmRun>())).io.direction = ((exit_qualification & 8) != 0) as u8;
        (*(vcpuinner.run.as_mut_ptr::<RkvmRun>())).io.count = 1;
        (*(vcpuinner.run.as_mut_ptr::<RkvmRun>())).exit_reason =
            (RkvmUserExitReason::from(exit_info.exit_reason)) as u32;
    }

    rkvm_debug!(
        " handle_io port ={:x} \n",
        (exit_qualification >> 16) as u16
    );

    exit_info.next_rip();
    Ok(0)
}

const LEVELBITS: u64 = 9;
macro_rules! LEVEL_SHIFT {
    ($level:expr) => {
        (bindings::PAGE_SHIFT as u64 + ($level - 1) * LEVELBITS)
    };
}

macro_rules! SHADOW_PT_INDEX {
    ($addr:expr, $level:expr) => {
        (($addr) >> LEVEL_SHIFT!($level) & ((1 << LEVELBITS) - 1))
    };
}

macro_rules! SPTE_TO_PFN {
    ($pte:expr) => {
       (($pte) & (bindings::physical_mask & ~((PAGE_SIZE - 1) as u64)))
    };
}
macro_rules! RKVM_PAGES_PER_HPAGE {
    ($level: expr) => {
        ((1 << ((($level) - 1) * 9 + bindings::PAGE_SHIFT as u64)) / PAGE_SIZE)
    };
}
fn rkvm_pagefault(vcpu: &VcpuWrapper, fault: &mut RkvmPageFault) -> Result {
    let vcpuinner = vcpu.vcpuinner.lock();
    let slot = &vcpuinner.guest.guestinner.lock().memslot;
    let uaddr = slot.userspace_addr;
    let base_gfn = slot.base_gfn;
    if fault.gfn < base_gfn {
        return Err(Error::EINVAL);
    }
    fault.hva = uaddr + (fault.gfn - base_gfn) * kernel::PAGE_SIZE as u64;
    let flags: u32 = bindings::FOLL_HWPOISON | bindings::FOLL_NOWAIT;

    let mut nrpages: i64 = 0;
    let mut pages = MaybeUninit::<*mut bindings::page>::uninit();
    unsafe {
        nrpages = bindings::get_user_pages_unlocked(uaddr, 1, pages.as_mut_ptr(), flags);
        if nrpages != 1 {
            return Err(Error::ENOMEM);
        }
        let page = *pages.as_mut_ptr() as *mut bindings::page;
        fault.pfn = bindings::rkvm_page_to_pfn(page);
        fault.goal_level = 1;
    }

    rkvm_debug!("pagefault: pfn={:?} \n", fault.pfn);

    Ok(())
}

fn rkvm_read_spte(mmu_page: Ref<RkvmMmuPage>, gfn: u64, level: u64) -> Result<u64> {
    if level < 1 {
        pr_err!(" rkvm_read_spte level={:?} < 1 \n", level);
        return Err(Error::EINVAL);
    }
    let offset: usize = SHADOW_PT_INDEX!((gfn << bindings::PAGE_SHIFT), level) as usize;
    let mut spte: u64 = 0;
    let p = &mut spte;
    unsafe {
        let ptr = core::slice::from_raw_parts_mut((p as *mut u64) as *mut u8, 8);
        mmu_page.pages.read(ptr.as_mut_ptr(), offset, 8);
    }
    Ok(spte)
}

fn rkvm_write_spte(mmu_page: Ref<RkvmMmuPage>, new_spte: u64, gfn: u64, level: u64) -> Result {
    if level < 1 {
        pr_err!(" rkvm_write_spte level={:?} < 1 \n", level);
        return Err(Error::EINVAL);
    }
    let offset: usize = SHADOW_PT_INDEX!((gfn << bindings::PAGE_SHIFT), level) as usize;
    let p = &new_spte;
    unsafe {
        let ptr = core::slice::from_raw_parts((p as *const u64) as *const u8, 8);
        mmu_page.pages.write(ptr.as_ptr(), offset, 8);
    }
    Ok(())
}

fn make_level_gfn(gfn: u64, level: u64) -> Result<u64> {
    if gfn == 0 {
        return Ok(0);
    }
    if level < 1 {
        pr_err!(" make_level_gfn: level={:?} < 1 \n", level);
        return Err(Error::EINVAL);
    }
    let level_gfn = (gfn + 1) & (-1 * RKVM_PAGES_PER_HPAGE!(level) as i64) as u64;
    Ok(level_gfn)
}

fn make_spte(fault: &RkvmPageFault, flags: &EptMasks) -> u64 {
    let pfn = fault.pfn;
    let mut spte: u64 = 1u64 << 11; //SPTE_MMU_PRESENT_MASK
    let pa = pfn << bindings::PAGE_SHIFT;

    if flags.ad_disabled {
        spte |= SpteFlag::SPTE_TDP_AD_DISABLED_MASK as u64;
    }
    // TODO: case with pte_access and marco ACC_XXXX_MASK
    // TODO: works related to mtrr & mmio
    spte |= pa | flags.ept_present_mask | flags.ept_exec_mask | flags.ept_user_mask;

    if !fault.prefetch && !flags.ad_disabled {
        spte |= flags.ept_accessed_mask;
    }

    //if host_writable
    spte |= SpteFlag::EPT_SPTE_HOST_WRITABLE as u64
        | SpteFlag::EPT_SPTE_MMU_WRITABLE as u64
        | VmxEptFlag::VMX_EPT_WRITABLE_MASK as u64;

    if !flags.ad_disabled {
        spte |= flags.ept_dirty_mask;
    }
    //TODO: permission settings in pte
    // spte |= pa | 0x77 | 0x600000000000000;
    spte
}

fn make_noleaf_spte(pt: u64, flags: &EptMasks) -> u64 {
    let mut spte: u64 = 1u64 << 11; //SPTE_MMU_PRESENT_MASK
    let pa = unsafe { bindings::rkvm_phy_address(pt) };

    spte |= pa
        | flags.ept_present_mask
        | flags.ept_user_mask
        | flags.ept_exec_mask
        | VmxEptFlag::VMX_EPT_WRITABLE_MASK as u64;

    if flags.ad_disabled {
        spte |= SpteFlag::SPTE_TDP_AD_DISABLED_MASK as u64;
    } else {
        spte |= flags.ept_accessed_mask;
    }

    //TODO: permission settings in pte
    // spte |= pa | 0x7u64;
    spte
}
fn rkvm_tdp_map(vcpu: &VcpuWrapper, fault: &mut RkvmPageFault) -> Result {
    let mut level: u64 = 4;
    let mut vcpuinner = vcpu.vcpuinner.lock();
    let level_gfn = make_level_gfn(fault.gfn, level)?;
    let mut pre_mmu_page = vcpuinner.mmu.root_mmu_page.clone();
    let flags = vcpuinner.mmu.spte_flags.clone();
    let mut spte = rkvm_read_spte(pre_mmu_page.clone(), level_gfn, level)?;
    while level > 0 {
        if level == fault.goal_level {
            break;
        }
        if !vcpuinner.mmu.is_pte_present(spte) {
            let mmu_page = vcpuinner.mmu.alloc_mmu_page(level - 1, level_gfn)?;
            let child_spt = match mmu_page.spt {
                Some(spt) => spt,
                None => return Err(Error::ENOMEM),
            };
            spte = make_noleaf_spte(child_spt, &flags);
            rkvm_write_spte(pre_mmu_page.clone(), spte, level_gfn, level - 1);

            rkvm_debug!(
                "rkvm_tdp_map level={:?}, gfn={:x}, spte={:x} \n",
                level,
                level_gfn,
                spte
            );

            pre_mmu_page = mmu_page;
        }
        level -= 1;
        let level_gfn = make_level_gfn(fault.gfn, level)?;
        spte = rkvm_read_spte(pre_mmu_page.clone(), level_gfn, level)?;
    } //while
      // handle leaf pte

    if level == fault.goal_level {
        //make pte
        spte = make_spte(fault, &flags);

        rkvm_debug!(
            "rkvm_tdp_map level={:?}, gfn={:x}, spte={:x} \n",
            level,
            level_gfn,
            spte
        );
        //set pte
        rkvm_write_spte(pre_mmu_page, spte, level_gfn, level);
    }
    Ok(())
}

pub(crate) fn handle_ept_violation(exit_info: &ExitInfo, vcpu: &VcpuWrapper) -> Result<u64> {
    rkvm_debug!("Enter handle EPT violation\n");

    let mut error_code: u64 = 0;
    let gpa = vmcs_read64(VmcsField::GUEST_PHYSICAL_ADDRESS);
    if (exit_info.exit_qualification & EptViolationMask::EPT_VIOLATION_ACC_READ as u64) != 0 {
        error_code = PferrMask::PFERR_USER_MASK as u64;
    }
    if (exit_info.exit_qualification & EptViolationMask::EPT_VIOLATION_ACC_WRITE as u64) != 0 {
        error_code |= PferrMask::PFERR_WRITE_MASK as u64;
    }
    if (exit_info.exit_qualification & EptViolationMask::EPT_VIOLATION_ACC_INSTR as u64) != 0 {
        error_code |= PferrMask::PFERR_FETCH_MASK as u64;
    }
    if exit_info.exit_qualification
        & (EptViolationMask::EPT_VIOLATION_READABLE as u64
            | EptViolationMask::EPT_VIOLATION_WRITABLE as u64
            | EptViolationMask::EPT_VIOLATION_EXECUTABLE as u64)
        != 0
    {
        error_code |= PferrMask::PFERR_PRESENT_MASK as u64;
    }
    if exit_info.exit_qualification & EptViolationMask::EPT_VIOLATION_GVA_TRANSLATED as u64 != 0 {
        error_code |= PferrMask::PFERR_GUEST_FINAL_MASK as u64;
    } else {
        error_code |= PferrMask::PFERR_GUEST_PAGE_MASK as u64;
    }

    let mut fault = RkvmPageFault {
        addr: gpa,
        error_code: error_code,
        present: (error_code & PferrMask::PFERR_PRESENT_MASK as u64) != 0,
        prefetch: false,
        gfn: gpa >> 12,
        pfn: 0,
        hva: 0,
        goal_level: 0,
    };

    let ret = rkvm_pagefault(vcpu, &mut fault);
    match ret {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    //map
    rkvm_tdp_map(vcpu, &mut fault);
    match ret {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    if invept(InvEptType::Global, 0).is_err() {
        pr_info!(" invept:Global failed\n");
    }
    Ok(1)
}

pub(crate) fn handle_cpuid(exit_info: &ExitInfo, vcpu: &VcpuWrapper) -> Result<u64> {
    rkvm_debug!("Enter cpuid exit handler\n");

    let mut vcpuinner = vcpu.vcpuinner.lock();
    // TODO : Capability check
    let irax = vcpuinner.guest_state.rax as u64;
    let ircx = vcpuinner.guest_state.rcx as u64;
    let (rax, rbx, rcx, rdx): (u64, u64, u64, u64);

    unsafe {
        asm!("cpuid\n\t", "mov rsi, rbx\n\t", inlateout("rax") irax => rax, lateout("rsi") rbx, inlateout("rcx") ircx => rcx, lateout("rdx") rdx);
    }

    rkvm_debug!(
        "emulate cpuid : rax= {:x}, rbx = {:x}, rcx = {:x}, rdx= {:x}\n",
        rax,
        rbx,
        rcx,
        rdx
    );

    vcpuinner.guest_state.rax = rax;
    vcpuinner.guest_state.rbx = rbx;
    vcpuinner.guest_state.rcx = rcx;
    vcpuinner.guest_state.rdx = rdx;

    exit_info.next_rip();

    Ok(0)
}
