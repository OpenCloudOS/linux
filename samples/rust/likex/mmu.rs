// SPDX-License-Identifier: GPL-2.0
use kernel::{
    bindings,
    c_types::c_void,
    linked_list::{GetLinks, Links, List},
    pages::Pages,
    prelude::*,
    sync::{Ref, UniqueRef},
    Result, PAGE_SIZE,
};

use crate::vmcs::*;
use crate::x86reg::*;
use crate::{rkvm_debug, DEBUG_ON};

#[repr(u64)]
#[derive(Debug)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum VmxEptpFlag {
    VMX_EPTP_PWL_MASK = 0x38,
    VMX_EPTP_PWL_4 = 0x18,
    VMX_EPTP_PWL_5 = 0x20,
    VMX_EPTP_AD_ENABLE_BIT = (1 << 6),
    VMX_EPTP_MT_MASK = 0x7,
    VMX_EPTP_MT_WB = 0x6,
    VMX_EPTP_MT_UC = 0x0,
}

#[repr(u64)]
#[derive(Debug)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub(crate) enum VmxEptFlag {
    VMX_EPT_READABLE_MASK = 0x1,
    VMX_EPT_WRITABLE_MASK = 0x2,
    VMX_EPT_EXECUTABLE_MASK = 0x4,
    VMX_EPT_IPAT_BIT = (1 << 6),
    VMX_EPT_ACCESS_BIT = (1 << 8),
    VMX_EPT_DIRTY_BIT = (1 << 9),
}

#[repr(u64)]
#[derive(Debug)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub(crate) enum SpteFlag {
    SPTE_TDP_AD_MASK = (3 << 52),
    SPTE_TDP_AD_ENABLED_MASK = (0 << 52),
    SPTE_TDP_AD_DISABLED_MASK = (1 << 52),
    SPTE_TDP_AD_WRPROT_ONLY_MASK = (2 << 52),
    EPT_SPTE_HOST_WRITABLE = (1 << 57),
    EPT_SPTE_MMU_WRITABLE = (1 << 58),
}

#[repr(u64)]
#[derive(Debug)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum VmxEptCapFlag {
    /* Appendix A.10  */
    VMX_EPT_EXECUTE_ONLY_BIT = 0x1,
    VMX_EPT_PAGE_WALK_4_BIT = (1 << 6),
    VMX_EPT_PAGE_WALK_5_BIT = (1 << 7),
    VMX_EPTP_UC_BIT = (1 << 8),
    VMX_EPTP_WB_BIT = (1 << 14),
    VMX_EPT_2MB_PAGE_BIT = (1 << 16),
    VMX_EPT_1GB_PAGE_BIT = (1 << 17),
    VMX_EPT_INVEPT_BIT = (1 << 20),
    VMX_EPT_AD_BIT = (1 << 21),
    VMX_EPT_EXTENT_CONTEXT_BIT = (1 << 25),
    VMX_EPT_EXTENT_GLOBAL_BIT = (1 << 26),
}

pub(crate) struct EptMasks {
    pub(crate) ept_user_mask: u64,
    pub(crate) ept_accessed_mask: u64,
    pub(crate) ept_dirty_mask: u64,
    pub(crate) ept_exec_mask: u64,
    pub(crate) ept_present_mask: u64,
    // ept_acc_track_mask: u64,
    pub(crate) ad_disabled: bool,
    pub(crate) has_exec_only: bool,
}

impl EptMasks {
    fn new() -> Result<Ref<Self>> {
        let ept_cap = read_msr(X86Msr::VMX_EPT_VPID_CAP);
        let user_mask = VmxEptFlag::VMX_EPT_READABLE_MASK as u64;
        let mut a_mask = VmxEptFlag::VMX_EPT_ACCESS_BIT as u64;
        let mut d_mask = VmxEptFlag::VMX_EPT_DIRTY_BIT as u64;
        let mut ad_disabled = false;
        if (ept_cap & VmxEptCapFlag::VMX_EPT_AD_BIT as u64) == 0 {
            a_mask = 0;
            d_mask = 0;
            ad_disabled = true;
        }
        let x_mask = VmxEptFlag::VMX_EPT_EXECUTABLE_MASK as u64;
        let mut present_mask = VmxEptFlag::VMX_EPT_READABLE_MASK as u64;
        let mut has_exec_only = false;
        if (ept_cap & VmxEptCapFlag::VMX_EPT_EXECUTE_ONLY_BIT as u64) != 0 {
            present_mask = 0;
            has_exec_only = true;
        }

        let pte_flags = Ref::try_new(Self {
            ept_user_mask: user_mask,
            ept_accessed_mask: a_mask,
            ept_dirty_mask: d_mask,
            ept_exec_mask: x_mask,
            ept_present_mask: present_mask,
            ad_disabled: ad_disabled,
            has_exec_only: has_exec_only,
        })?;

        Ok(pte_flags)
    }
}

pub(crate) struct RkvmMmu {
    pub(crate) root_hpa: u64,
    pub(crate) root_mmu_page: Ref<RkvmMmuPage>,
    mmu_root_list: List<Ref<RkvmMmuPage>>,
    mmu_pages_list: List<Ref<RkvmMmuPage>>,
    pub(crate) spte_flags: Ref<EptMasks>,
}

impl RkvmMmu {
    pub(crate) fn new() -> Result<UniqueRef<Self>> {
        let root = RkvmMmuPage::new(true, 4, None); //root level = 4
        let root = match root {
            Ok(root) => root,
            Err(err) => return Err(err),
        };

        let hpa = match root.spt {
            Some(hpa) => hpa,
            None => return Err(Error::ENOMEM),
        };

        rkvm_debug!("RkvmMmu hpa(va) = {:x} \n", hpa);

        let ptr = hpa as *mut c_void;
        unsafe {
            bindings::memset(ptr, 0, PAGE_SIZE as u64);
        }
        let hpa = unsafe { bindings::rkvm_phy_address(hpa) };

        rkvm_debug!("RkvmMmu hpa(phy) = {:x}--root_hpa \n", hpa);

        let flags = EptMasks::new();
        let flags = match flags {
            Ok(flags) => flags,
            Err(err) => return Err(err),
        };
        rkvm_debug!(
            "ad_disabled = {}, ecex_only = {}",
            flags.ad_disabled,
            flags.has_exec_only
        );

        let mut mmu = UniqueRef::try_new(Self {
            root_hpa: hpa, //physical addr
            root_mmu_page: root.clone(),
            mmu_root_list:  List::new(),
            mmu_pages_list: List::new(),
            spte_flags: flags.clone(),
        })?;

        mmu.mmu_root_list.push_back(root);
        Ok(mmu)
    }
    pub(crate) fn alloc_mmu_page(&mut self, level: u64, gfn: u64) -> Result<Ref<RkvmMmuPage>> {
        let mmu_page = RkvmMmuPage::new(false, level, Some(gfn));
        let mmu_page = match mmu_page {
            Ok(page) => page,
            Err(err) => return Err(err),
        };
        let vaddr = match mmu_page.spt {
            Some(va) => va,
            None => return Err(Error::ENOMEM),
        };
        let ptr = vaddr as *mut c_void;
        unsafe {
            bindings::memset(ptr, 0, PAGE_SIZE as u64);
        }
        let ret = mmu_page.clone();
        self.mmu_pages_list.push_back(mmu_page);
        Ok(ret)
    }

    pub(crate) fn make_eptp(&mut self) -> u64 {
        let mut eptp: u64 = VmxEptpFlag::VMX_EPTP_MT_WB as u64 | VmxEptpFlag::VMX_EPTP_PWL_4 as u64;
        eptp |= self.root_hpa; //| (1u64 << 6);
        eptp
    }

    pub(crate) fn init_mmu_root(&mut self) -> Result {
        rkvm_debug!(" init_mmu_root \n");

        let eptp = self.make_eptp();
        vmcs_write64(VmcsField::EPT_POINTER, eptp);

        rkvm_debug!("hpa= {:x}, eptp = {:x} \n", self.root_hpa, eptp);

        if invept(InvEptType::Single, eptp).is_err() {
            pr_info!(" invept:Single, eptp=0x{:x} failed \n", eptp);
        }
        Ok(())
    }
    pub(crate) fn is_pte_present(&self, pte: u64) -> bool {
        (pte & (1u64 << 11)) != 0u64
    }
}

#[allow(dead_code)]
pub(crate) struct RkvmMmuPage {
    pub(crate) gfn: Option<u64>,
    pub(crate) pages: Pages<0>,
    pub(crate) root: bool,
    pub(crate) spt: Option<u64>, //mmu page's vaddr
    pub(crate) level: u64,
    pub(crate) page_links: Links<RkvmMmuPage>,
}

impl RkvmMmuPage {
    pub(crate) fn new(isroot: bool, level: u64, gfn: Option<u64>) -> Result<Ref<Self>> {
        let page = Pages::<0>::new();
        let page = match page {
            Ok(page) => page,
            Err(err) => return Err(err),
        };
        let spt = unsafe { Some(bindings::page_address(page.pages) as u64) };

        let mmu_page = Ref::try_new(RkvmMmuPage {
            gfn: gfn,
            pages: page,
            root: isroot,
            spt: spt,
            level: level,
            page_links: Links::new(),
        })?;

        Ok(mmu_page)
    }
}

impl GetLinks for RkvmMmuPage {
    type EntryType = RkvmMmuPage;
    fn get_links(data: &RkvmMmuPage) -> &Links<RkvmMmuPage> {
        &data.page_links
    }
}
