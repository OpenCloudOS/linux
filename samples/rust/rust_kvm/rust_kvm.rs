// SPDX-License-Identifier: GPL-2.0

//! Rust KVM for VMX
//#![feature(asm)]
#[allow(dead_code)]
use kernel::prelude::*;
use kernel::{
    file::File,
    file_operations::{FileOperations, IoctlCommand, IoctlHandler},
    miscdev, 
    sync::{CondVar, Mutex, Ref, RefBorrow, UniqueRef},
    bindings,
    pages::Pages,
    bit,
};
mod guest;
mod vmcs;
mod x86;
use crate::x86::x86reg::Cr4;
use crate::guest::Guest;
use crate::vmcs::*;
module! {
    type: RustMiscdev,
    name: b"rust_kvm",
    author: b"Peng Hao",
    description: b"Rust KVM VMX",
    license: b"GPL v2",
}

struct SharedStateInner {
    token_count: usize,
}

struct RkvmState {
    vmcsconf: VmcsConfig,
    state_changed: CondVar,
    inner: Mutex<SharedStateInner>,
}

// used for vmxon
struct VmxInfo {
    revision_id: u32,
    region_size: u16,
    write_back: bool,
    io_exit_info: bool,
    vmx_controls: bool,
}

impl RkvmState {
    fn try_new() -> Result<Ref<Self>> {
        pr_info!("RkvmState try_new \n");
        let mut vmcsconf = VmcsConfig::new()?;
        vmcsconf.setup_config();
        let mut state = Pin::from(UniqueRef::try_new(Self {
            // SAFETY: `condvar_init!` is called below
            vmcsconf: vmcsconf,
            state_changed: unsafe { CondVar::new() },
            // SAFETY: `mutex_init!` is called below.
            inner: unsafe { Mutex::new(SharedStateInner { token_count: 0 }) },
        })?);

        // SAFETY: `state_changed` is pinned when `state` is.
        let pinned = unsafe { state.as_mut().map_unchecked_mut(|s| &mut s.state_changed) };
        kernel::condvar_init!(pinned, "RkvmState::state_changed");

        // SAFETY: `inner` is pinned when `state` is.
        let pinned = unsafe { state.as_mut().map_unchecked_mut(|s| &mut s.inner) };
        kernel::mutex_init!(pinned, "RkvmState::inner");

        Ok(state.into())
    }
}

struct KVM;
impl FileOperations for KVM {
    type Wrapper = Ref<RkvmState>;
    type OpenData = Ref<RkvmState>;

    kernel::declare_file_operations!(ioctl);

    fn open(shared: &Ref<RkvmState>, _file: &File) -> Result<Self::Wrapper> {
        pr_info!("KVM open \n");
        Ok(shared.clone())
    }

    fn ioctl(shared: RefBorrow<'_, RkvmState>, file: &File, cmd: &mut IoctlCommand) -> Result<i32> {
        cmd.dispatch::<RkvmState>(&shared, file)
    }
}

struct RustMiscdev {
    _dev: Pin<Box<miscdev::Registration<KVM>>>,
}

impl KernelModule for RustMiscdev {
    fn init(name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust kvm device init\n");

        let state = RkvmState::try_new()?;
        /* vmxon percpu*/

        Ok(RustMiscdev {
            _dev: miscdev::Registration::new_pinned(fmt!("{name}"), state)?,
        })
    }
}

impl Drop for RustMiscdev {
    fn drop(&mut self) {
        pr_info!("Rust kvm device sample (exit)\n");
    }
}

fn rkvm_set_vmxon(state: &RkvmState) -> Result<u32> {
    // allocate page for vmxon
    let page = Pages::<0>::new();

    let page = match page {
        Ok(page) => page,
        Err(err) => return Err(/*Error::ENOMEM*/ err),
    };

    let vmxinfo = VmxInfo {
        revision_id: state.vmcsconf.revision_id,
        region_size: state.vmcsconf.size as u16,
        write_back: false,
        io_exit_info: false,
        vmx_controls: true,
    };

    let mut kva: u64 = 0;
    unsafe {
        kva = bindings::rkvm_page_address(page.pages);
        let len = core::mem::size_of::<VmxInfo>();
        let p = &vmxinfo;
        pr_info!(
            "Rust kvm:kva={:x}, size={:?},revision={:?} \n",
            kva,
            state.vmcsconf.size,
            vmxinfo.revision_id
        );
        let ptr = core::slice::from_raw_parts((p as *const VmxInfo) as *const u8, len);

        page.write(ptr.as_ptr(), 0, len);
    }

    let mut vmxe: u64 = 0;
    unsafe {
        vmxe = bindings::native2_read_cr4() & bit(Cr4::CR4_ENABLE_VMX);
    }
    pr_info!("Rust kvm :vmxe {:}\n", vmxe);
    if vmxe > 0 {
        pr_info!("Rust kvm: vmx has been enabled\n");
        return Err(Error::ENOENT);
    }
    unsafe {
        let pa = bindings::rkvm_phy_address(kva);
        pr_info!(" pa = {:x}\n", pa);
        bindings::rkvm_vmxon(pa);
    }
    Ok(0)
}

const IOCTL_KVM_CREATE_VM: u32 = 0x00AE0100;
const IOCTL_KVM_CREATE_VCPU: u32 = 0x00AE4100;
static mut GUEST: Option<Ref<Mutex<Guest>>> = None;
impl IoctlHandler for RkvmState {
    type Target<'a> = &'a RkvmState;

    fn pure(_shared: &RkvmState, _: &File, cmd: u32, _arg: usize) -> Result<i32> {
        match cmd {
            IOCTL_KVM_CREATE_VM => {
                if let Err(error) = rkvm_set_vmxon(_shared) {
                    pr_err!("Rkvm: IOCTL_KVM_CREATE_VM failed\n");
                    return Err(error);
                }
                let guest = Guest::new();

                let guest = match guest {
                    Err(error) => return Err(error),
                    Ok(guest) => guest,
                };
                unsafe {
                    GUEST = Some(guest);
                }
                pr_info!("Rust kvm: IOCTL_KVM_CREATE_VM\n");
                Ok(0)
            }
            IOCTL_KVM_CREATE_VCPU => {
                pr_info!("Rust kvm: IOCTL_KVM_CREATE_VCPU\n");
                Ok(0)
            }
            _ => Err(Error::EINVAL),
        }
    }
}
