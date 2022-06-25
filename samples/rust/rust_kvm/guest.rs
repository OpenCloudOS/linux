// SPDX-License-Identifier: GPL-2.0
use kernel::prelude::*;
use kernel::task::Task;
use kernel::{bindings, mutex_init, Error, Result};
//use kernel::rbtree::{RBTree, RBTreeNode};
use kernel::sync::{Mutex, Ref, UniqueRef};
use crate::{rkvm_debug, DEBUG_ON};

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub(crate) struct RkvmMemorySlot {
    //pub(crate) gfn_node: RBTreeNode,
    pub(crate) base_gfn: u64,
    pub(crate) npages: u64,
    pub(crate) userspace_addr: u64,
    pub(crate) as_id: u16,
}
/*
pub(crate) struct Rkvm_memslots {
    pub(crate) gfn_tree: RBTree<u64,u64>,
    pub(crate) node_index: u64,
}
*/

#[allow(dead_code)]
pub(crate) struct Guest {
    pub(crate) mm: *const bindings::mm_struct,
    pub(crate) memslot: RkvmMemorySlot,
    pub(crate) nr_slot_pages: u64,
    //pub(crate) mmu: Rkvm_mmu,
}

pub(crate) struct GuestWrapper {
    pub(crate) guestinner: Mutex<Guest>,
}

impl GuestWrapper {
    /// Create a Guest.
    pub(crate) fn new() -> Result<Ref<Self>> {
        let mm_ = Task::current().mm();

        let mut g = Pin::from(UniqueRef::try_new(Self {
            guestinner: unsafe {
                Mutex::new(Guest {
                    mm: mm_,
                    memslot: RkvmMemorySlot {
                        base_gfn: 0,
                        npages: 0,
                        userspace_addr: 0,
                        as_id: 0,
                    },
                    nr_slot_pages: 0,
                })
            },
        })?);
        let pinned = unsafe { g.as_mut().map_unchecked_mut(|s| &mut s.guestinner) };
        mutex_init!(pinned, "GuestWrapper::guestinner");

        Ok(g.into())
    }

    pub(crate) fn add_memory_region(&self, uaddr: u64, npages: u64, gpa: u64) -> Result<i32> {
        if gpa & (kernel::PAGE_SIZE - 1) as u64 != 0 {
            return Err(Error::ENOMEM);
        }
        let mut guestinner = self.guestinner.lock();
        guestinner.memslot.userspace_addr = uaddr;
        guestinner.memslot.base_gfn = gpa >> 12;
        guestinner.memslot.npages = npages;

        rkvm_debug!(
            " add_memory_region user={:x}, gpa = {:x}, npages={:x} \n",
            uaddr,
            gpa,
            npages
        );
        
        Ok(0)
    }
}

impl Drop for GuestWrapper {
    fn drop(&mut self) {
        rkvm_debug!(" guest droped \n");
    }
}
