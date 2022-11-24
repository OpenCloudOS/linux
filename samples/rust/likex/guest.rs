// SPDX-License-Identifier: GPL-2.0
use kernel::prelude::*;
use kernel::task::Task;
use kernel::{bindings, mutex_init, Result};
use kernel::unsafe_list::{Adapter, Links, List};
use crate::{rkvm_debug, DEBUG_ON};
use kernel::sync::{Mutex, Arc, UniqueArc};
use kernel::PAGE_SIZE;
//#[derive(Copy, Clone)]
#[allow(dead_code)]
pub(crate) struct RkvmMemorySlot {
    pub(crate) links: Links<RkvmMemorySlot>,
    pub(crate) base_gfn: u64,
    pub(crate) npages: u64,
    pub(crate) userspace_addr: u64,
    pub(crate) slot_id: u16,
}

unsafe impl Adapter for RkvmMemorySlot {
    type EntryType = Self;
    fn to_links(obj: &Self) -> &Links<Self> {
       &obj.links
    }
}


#[allow(dead_code)]
pub(crate) struct Guest {
    pub(crate) mm: *const bindings::mm_struct,
    pub(crate) slots_list: List<RkvmMemorySlot>,
    pub(crate) num_slots: u64,
}

pub(crate) struct GuestWrapper {
    pub(crate) guestinner: Mutex<Guest>,
}

impl GuestWrapper {
    /// Create a Guest.
    pub(crate) fn new() -> Result<Arc<Self>> {
        let mm_ = unsafe { 
                    Task::current().mm().get()
                  };

        let mut guest = Pin::from(UniqueArc::try_new(Self {
            guestinner: unsafe {
                Mutex::new(Guest {
                    mm: mm_,
                    slots_list: List::<RkvmMemorySlot>::new(),
                    num_slots: 0,
                })
            },
        })?);
        let pinned = unsafe { guest.as_mut().map_unchecked_mut(|s| &mut s.guestinner) };
        mutex_init!(pinned, "GuestWrapper::guestinner");

        Ok(guest.into())
    }

    pub(crate) fn add_memory_region(&self, slot: u16, uaddr: u64, npages: u64, gpa: u64) -> Result<i32> {
        if gpa & (kernel::PAGE_SIZE - 1) as u64 != 0 {
            return Err(ENOMEM);
        }
        let newslot = UniqueArc::try_new(RkvmMemorySlot {
                           links: Links::new(),
                           base_gfn: gpa >> 12,
                           npages: npages,
                           userspace_addr: uaddr,
                           slot_id: slot,
                      })?;
        let newslot = Arc::from(newslot);

        // TODO: Dealing with slot overlap issues
        let mut guestinner = self.guestinner.lock();

        // Add one reference into a pointer to hold on to a ref count while the
        // slot is in the list.
        Arc::into_raw(newslot.clone());
        unsafe { guestinner.slots_list.push_back(&*newslot) };
        guestinner.num_slots += 1;
        rkvm_debug!(
            " add_memory_region slot= {},uaddr={:x}, gpa = {:x}, npages={:x} \n",
	    slot,
            uaddr,
            gpa,
            npages
        );

        Ok(0)
    }

    pub(crate) fn find_slot(&self, gfn: u64) -> Result<Arc<RkvmMemorySlot>> {
       let guestinner = self.guestinner.lock();
       for (_i, e) in guestinner.slots_list.iter().enumerate() {
           if (gfn >= e.base_gfn) && (gfn <= e.base_gfn + PAGE_SIZE as u64 * e.npages) {
              let slot = unsafe { Arc::<RkvmMemorySlot>::from_raw(e) };
              return Ok(slot);
           }
       }
       Err(EINVAL)
   }
}

impl Drop for GuestWrapper {
    fn drop(&mut self) {
        pr_info!(" guest droped \n");
    }
}
