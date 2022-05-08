// SPDX-License-Identifier: GPL-2.0
use kernel::prelude::*;
use kernel::task::{Task, Mm};
use kernel::{Error, Result};
//use kernel::rbtree::{RBTree, RBTreeNode};
use kernel::sync::{Mutex, Ref};
use kernel::bindings;
use core::ptr;
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
    //pub(crate) mmu: Rkvm_mmu,
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
            }))?;
        }
        Ok(g)
    }
    pub(crate) fn mmu_init(&self) {
        //ept init
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
