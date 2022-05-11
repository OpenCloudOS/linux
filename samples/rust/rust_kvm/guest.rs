// SPDX-License-Identifier: GPL-2.0
use kernel::prelude::*;
use kernel::task::{Task, Mm, Pid};
use kernel::{Error, Result};
//use kernel::rbtree::{RBTree, RBTreeNode};
use kernel::sync::{Mutex, Ref};
use kernel::bindings;
use core::ptr;
use crate::x86::mmu::RkvmMmu;

#[derive(Copy, Clone)]
pub(crate) struct RkvmMemorySlot {
    //pub(crate) gfn_node: RBTreeNode,
    pub(crate) base_gfn: u64,
    pub(crate) npages: u64,
    pub(crate) userspace_addr: u64,
    pub(crate) as_id: u16,
    pub(crate) base_hva: u64,
}

pub(crate) struct Guest {
    pub(crate) mm: Mm,
    pub(crate) memslot: RkvmMemorySlot,
    pub(crate) nr_slot_pages: u64,
    pub(crate) mmu: RkvmMmu,
}

pub(crate) struct RkvmVCPU {
    rkvm: Ref<Guest>,

    cpu: u32,
    vcpu_id: u32,
    mode: u32,
    requests: u64,

    // struct mutex mutex;
	// struct kvm_run *run;

    pid: Pid,

    ready: bool,
    // arch: RkvmVCPUArch,
    // stat: RkvmVCPUSTAT,

}

impl Guest {
    /// Create a Guest.
    pub(crate) fn new() -> Result<Ref<Mutex<Self>>> {
        let mm_ = Task::current().mm();

        let g;
        unsafe {
            g = Ref::try_new(Mutex::new(Self {
                mm: mm_,
                memslot: RkvmMemorySlot {
                    base_gfn: 0,
                    npages: 0,
                    userspace_addr: 0,
                    as_id: 0,
                    base_hva: 0,
                },
                nr_slot_pages: 0,
                mmu: RkvmMmu {
                    root_hpa: None,
                    root_pgd: None,
                    mmu_role: 0,
                    root_level: 0,
                    tdp_root_level: 0,
                    ept_ad: 0,
                    direct_map: false,
                    pae_root: None,
                    pml4_root: None,
                    pml5_root: None,
                }
            }))?;
        }
        Ok(g)
    }

    pub(crate) fn mmu_init(&mut self) {
        // ept init
        // Reference to init_kvm_tdp_mmu()
        
        // releated to architecture, solve it later
        /*
        struct kvm_mmu_role_regs regs = vcpu_to_role_regs(vcpu);
        union kvm_mmu_role new_role =
            kvm_calc_tdp_mmu_root_page_role(vcpu, &regs, false);
    
        if (new_role.as_u64 == context->mmu_role.as_u64)
            return;
        */    
        let new_role: u64 = 0; // rm when solve the codes above

        self.mmu.mmu_role = new_role;
        self.mmu.tdp_root_level = 4;
        self.mmu.root_level = 4; // set to 5 if host enable 5-level paging
        self.mmu.direct_map = true;
    }

    pub(crate) fn add_memory_region(&mut self, uaddr: u64, npages: u64, gpa: u64) -> Result<u32> {
        if gpa & (kernel::PAGE_SIZE - 1) as u64 != 0 {
            return Err(Error::ENOMEM);
        }
        self.memslot.userspace_addr = uaddr;
        self.memslot.base_gfn = gpa >> 12u32;
        self.memslot.npages = npages;

        // linear mapping from gpa->hva
        self.memslot.base_hva = unsafe {
            bindings::krealloc(ptr::null(), (npages * (1 << 12u32)).try_into().unwrap(), bindings::GFP_KERNEL) as u64 };

        Ok(0)
    }
}
