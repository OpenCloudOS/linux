// SPDX-License-Identifier: GPL-2.0

use kernel::{Error, Result};
use kernel::linked_list::List;
use alloc::boxed::Box;
use crate::guest::RkvmMemorySlot;
use kernel::sync::{Mutex, Ref};

pub(crate) struct RkvmMmuPage {
    // link: List<Box<u32>>, //?
    // struct hlist_node hash_link;

    tdp_mmu_page: bool,
    unsync: bool,
    mmu_valid_gen: u8,
    // bool lpage_disallowed;

    // union kvm_mmu_page_role role;
    mmu_page_role: u64,

    gfn: u64,
    spt: *const u64,
    gfns: *const u64,

    // union {
	// 	int root_count;
	// 	refcount_t tdp_mmu_root_count;
	// };
    // unsigned int unsync_children;
    // DECLARE_BITMAP(unsync_child_bitmap, 512);

	// struct list_head lpage_disallowed_link;
    // atomic_t write_flooding_count;
    // struct rcu_head rcu_head;
}

pub(crate) struct RkvmMmu {
    pub(crate) root_hpa: Option<u64>,
    pub(crate) root_pgd: Option<u64>,
    
    pub(crate) mmu_role: u64,
    
    pub(crate) root_level: u8,
    pub(crate) tdp_root_level: u8,
    pub(crate) ept_ad: u8,
    pub(crate) direct_map: bool,
    // struct kvm_mmu_root_info prev_roots[KVM_MMU_NUM_PREV_ROOTS];
    // u8 permissions[16];

    // pkru_mask: u32,

    pub(crate) pae_root: Option<*const u64>,
    pub(crate) pml4_root: Option<*const u64>,
    pub(crate) pml5_root: Option<*const u64>,

    // struct rsvd_bits_validate shadow_zero_check;
	// struct rsvd_bits_validate guest_rsvd_check;
	// u64 pdptrs[4]; /* pae */
}

pub(crate) struct RkvmPageFault {
    /* arguments for kvm_mmu_do_page_fault */
    addr: u64,  // gpa
    error_code: u32,
    prefetch: bool,

    /* derived from error code */
    exec: bool,
    write: bool, 
    present: bool,
    rsvd: bool,
    user: bool,

    max_level: u8,
    req_level: u8,
    goal_level: u8,

    gfn: u64,
    
    slot: Ref<Mutex<RkvmMemorySlot>>,

    /* Outputs of kvm_faultin_pfn.  */
	// kvm_pfn_t pfn;
	// hva_t hva;
	// bool map_writable;
}

impl RkvmMmu {

    fn get_guest_pgd(&self) -> Result<u64> {
        Ok(0)
    }

    fn get_pdptr(&self) -> Result<u64> {
        Ok(0)
    }

    fn page_fault(&self) -> Result<u32> {
        Ok(0)
    }

    fn inject_page_fault(&self) -> Result<u8> {
        Ok(0)
    }

    fn gva_to_gpa(&self) -> Result<u64> {
        Ok(0)
    }

    fn sync_page(&self) -> Result<u32> {
        Ok(0)
    }

    fn invlpg(&self) -> Result<u8> {
        Ok(0)
    }

}