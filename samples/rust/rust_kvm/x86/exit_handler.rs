// SPDX-License-Identifier: GPL-2.0

use kernel::{Error, Result};
use kernel::sync::{Mutex, Ref};
use crate::guest::RkvmVCPU;

pub(crate) struct RkvmExit {
    exit_reason: u32, // union in linux kvm, basic in higher 16 bits
    /*
        EXIT_REASON_EXCEPTION_NMI       0
        EXIT_REASON_EXTERNAL_INTERRUPT  1
        EXIT_REASON_TRIPLE_FAULT        2
        EXIT_REASON_INIT_SIGNAL			3
        EXIT_REASON_SIPI_SIGNAL         4
        EXIT_REASON_INTERRUPT_WINDOW    7
        EXIT_REASON_NMI_WINDOW          8
        EXIT_REASON_TASK_SWITCH         9
        EXIT_REASON_CPUID               10
        EXIT_REASON_HLT                 12
        EXIT_REASON_INVD                13
        EXIT_REASON_INVLPG              14
        EXIT_REASON_RDPMC               15
        EXIT_REASON_RDTSC               16
        EXIT_REASON_VMCALL              18
        EXIT_REASON_VMCLEAR             19
        EXIT_REASON_VMLAUNCH            20
        EXIT_REASON_VMPTRLD             21
        EXIT_REASON_VMPTRST             22
        EXIT_REASON_VMREAD              23
        EXIT_REASON_VMRESUME            24
        EXIT_REASON_VMWRITE             25
        EXIT_REASON_VMOFF               26
        EXIT_REASON_VMON                27
        EXIT_REASON_CR_ACCESS           28
        EXIT_REASON_DR_ACCESS           29
        EXIT_REASON_IO_INSTRUCTION      30
        EXIT_REASON_MSR_READ            31
        EXIT_REASON_MSR_WRITE           32
        EXIT_REASON_INVALID_STATE       33
        EXIT_REASON_MSR_LOAD_FAIL       34
        EXIT_REASON_MWAIT_INSTRUCTION   36
        EXIT_REASON_MONITOR_TRAP_FLAG   37
        EXIT_REASON_MONITOR_INSTRUCTION 39
        EXIT_REASON_PAUSE_INSTRUCTION   40
        EXIT_REASON_MCE_DURING_VMENTRY  41
        EXIT_REASON_TPR_BELOW_THRESHOLD 43
        EXIT_REASON_APIC_ACCESS         44
        EXIT_REASON_EOI_INDUCED         45
        EXIT_REASON_GDTR_IDTR           46
        EXIT_REASON_LDTR_TR             47
        EXIT_REASON_EPT_VIOLATION       48
        EXIT_REASON_EPT_MISCONFIG       49
        EXIT_REASON_INVEPT              50
        EXIT_REASON_RDTSCP              51
        EXIT_REASON_PREEMPTION_TIMER    52
        EXIT_REASON_INVVPID             53
        EXIT_REASON_WBINVD              54
        EXIT_REASON_XSETBV              55
        EXIT_REASON_APIC_WRITE          56
        EXIT_REASON_RDRAND              57
        EXIT_REASON_INVPCID             58
        EXIT_REASON_VMFUNC              59
        EXIT_REASON_ENCLS               60
        EXIT_REASON_RDSEED              61
        EXIT_REASON_PML_FULL            62
        EXIT_REASON_XSAVES              63
        EXIT_REASON_XRSTORS             64
        EXIT_REASON_UMWAIT              67
        EXIT_REASON_TPAUSE              68
        EXIT_REASON_BUS_LOCK            74
    */
}

impl RkvmExit {
    pub(crate) fn new() -> Result<Self> {
        //vmx->exit_reason;
        let h = Self {
            exit_reason: 0,
        };

        Ok(h)
    }

    pub(crate) fn handler(&self, vcpu: &RkvmVCPU) -> Result<u8>{
        let mut r: u8 = 0;

        match self.exit_reason {
            48 => r = self.ept_violation_handler(vcpu)?,
            _ => self.unexpected_vmexit(),
        }

        Ok(r)
    }

    fn ept_violation_handler(&self, vcpu: &RkvmVCPU) -> Result<u8> {
        let r: u8 = 0;

        Ok(r)
    }

    fn unexpected_vmexit(&self) {
        panic!("Not a valid vmexit exit source");
    }
}