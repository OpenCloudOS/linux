// SPDX-License-Identifier: GPL-2.0

use kernel::prelude::*;

pub(crate) struct RkvmIodevice {}

pub(crate) struct LapicReg {}

pub(crate) struct RkvmLapic {
    pub(crate) base_address: u64,
    pub(crate) dev: RkvmIodevice,
    pub(crate) lapic_timer: RkvmTimer,
    pub(crate) sw_enabled: bool,
    pub(crate) irr_pending: bool,

    /// The highest vector set in ISR; if -1 - invalid, must scan ISR.
    pub(crate) highest_isr_cache: u32,
    /// APIC register page.  The layout matches the register layout seen by
    /// the guest 1:1, because it is accessed by the vmx microcode.
    /// Note: Only one register, the TPR, is used by the microcode.
    regs: LapicReg,
    pub(crate) vapic_addr: u64,
}
