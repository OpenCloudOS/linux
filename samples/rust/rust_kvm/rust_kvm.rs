// SPDX-License-Identifier: GPL-2.0

//! Rust KVM for VMX
#[allow(missing_docs)]
use core::cfg;
use core::arch::asm;
use kernel::c_types::c_void;
use kernel::mm::virt::Area;
use kernel::prelude::*;
use kernel::{
    bindings, bit, Error,
    file::File,
    file_operations::{FileOperations, IoctlCommand, IoctlHandler},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev,
    sync::{CondVar, Mutex, Ref, RefBorrow, UniqueRef},
    user_ptr::{UserSlicePtrReader, UserSlicePtrWriter},
    Result,
};

mod exit;
mod guest;
mod mmu;
mod vcpu;
mod vmcs;
mod vmstat;
mod x86reg;

use crate::guest::GuestWrapper;
use crate::vcpu::*;
use crate::vmcs::*;
use crate::x86reg::*;

module! {
    type: RustMiscdev,
    name: b"rust_kvm",
    author: b"Peng Hao",
    description: b"Rust KVM VMX",
    license: b"GPL v2",
    params: {
        debug: bool {
            default: true,
            permissions: 0,
            description: b"debug print control",
        },
    },
}

#[allow(dead_code)]
struct SharedStateInner {
    token_count: usize,
    vmcsconf: VmcsConfig,
}

#[allow(dead_code)]
struct RkvmState {
    state_changed: CondVar,
    inner: Mutex<SharedStateInner>,
}

impl RkvmState {
    fn try_new() -> Result<Ref<Self>> {
        rkvm_debug!("RkvmState try_new \n");
        let mut vmcsconf = VmcsConfig::new()?;
        let _ret = vmcsconf.setup_config()?;
        let mut state = Pin::from(UniqueRef::try_new(Self {
            // SAFETY: `condvar_init!` is called below.
            state_changed: unsafe { CondVar::new() },
            // SAFETY: `mutex_init!` is called below.
            inner: unsafe {
                Mutex::new(SharedStateInner {
                    token_count: 0,
                    vmcsconf: vmcsconf,
                })
            },
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

    kernel::declare_file_operations!(ioctl, mmap);

    fn open(shared: &Ref<RkvmState>, _file: &File) -> Result<Self::Wrapper> {
        rkvm_debug!("KVM open \n");

        Ok(shared.clone())
    }

    fn mmap(_shared: RefBorrow<'_, RkvmState>, _file: &File, _vma: &mut Area) -> Result {
        rkvm_debug!("KVM mmap \n");

        unsafe {
            bindings::rkvm_mmap(_file.ptr, _vma.vma);
        }
        Ok(())
    }

    fn ioctl(shared: RefBorrow<'_, RkvmState>, file: &File, cmd: &mut IoctlCommand) -> Result<i32> {
        cmd.dispatch::<RkvmState>(&shared, file)
    }
}

static mut DEBUG_ON: bool = false;
#[macro_export]
/// debug print switch
macro_rules! rkvm_debug (
    ($($arg:tt)*) => (
        unsafe {
            if DEBUG_ON {
                kernel::print_macro!(kernel::print::format_strings::DEBUG, false, $($arg)*)
            }
        }
    )
);

struct RustMiscdev {
    _dev: Pin<Box<miscdev::Registration<KVM>>>,
}

impl KernelModule for RustMiscdev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        if cfg!(CONFIG_X86_32) {
            pr_err!("RustKvm doesn't support 32-bit machine\n");
            return Err(Error::ENOTSUPP);
        }
        
        pr_info!("Rust kvm module (init) name={:?} \n", name);
        {
            let _lock = module.kernel_param_lock();
            unsafe {
                DEBUG_ON = *debug.read();
            }
            rkvm_debug!("DEBUG ON :  {}\n", DEBUG_ON);
        }
        let state = RkvmState::try_new()?;
        Ok(RustMiscdev {
            _dev: miscdev::Registration::new_pinned(fmt!("{name}"), state)?,
        })
    }
}

impl Drop for RustMiscdev {
    fn drop(&mut self) {
        //unsafe { bindings::rkvm_vmxoff();}
        pr_info!("Rust kvm module (exit)\n");
    }
}

static mut VMXON_VMCS: Option<RkvmPage> = None;
fn rkvm_vmxon(addr: u64) {
    let mut cr4: u64 = read_cr4();
    cr4 |= Cr4::CR4_ENABLE_VMX as u64;
    write_cr4(cr4);
    rkvm_debug!("written new cr4 value\n");
    unsafe {
        asm!(
            "vmxon ({0})",
            in(reg) &addr,
            options(att_syntax)
        );
    }
}

fn rkvm_set_vmxon(state: &RkvmState) -> Result<u32> {
    let revision_id = state.inner.lock().vmcsconf.revision_id;
    let vmcs = alloc_vmcs(revision_id);
    let vmcs = match vmcs {
        Ok(vmcs) => vmcs,
        Err(err) => return Err(/*Error::ENOMEM*/ err),
    };

    let vmxe = read_cr4() & Cr4::CR4_ENABLE_VMX as u64;

    rkvm_debug!("Rust kvm :vmxe {:}\n", vmxe);

    if vmxe > 0 {
        pr_err!("Rust kvm: vmx has been enabled\n");
        return Err(Error::ENOENT);
    }
    unsafe {
        let pa = bindings::rkvm_phy_address(vmcs.va);

        rkvm_debug!(" pa = {:x}\n", pa);

        rkvm_vmxon(pa);
        VMXON_VMCS = Some(vmcs);
    }
    Ok(0)
}

const IOCTL_KVM_CREATE_VM: u32 = 0x0000AE01;
const IOCTL_KVM_CREATE_VCPU: u32 = 0x0000AE41;
const IOCTL_KVM_VCPU_RUN: u32 = 0x0000AE80;
const IOCTL_KVM_SET_USER_MEMORY_REGION: u32 = 0x4020AE46;
const IOCTL_KVM_GET_REGS: u32 = 0x8090AE81;
const IOCTL_KVM_SET_REGS: u32 = 0x4090AE82;
const IOCTL_KVM_GET_SREGS: u32 = 0x8138AE83;
const IOCTL_KVM_SET_SREGS: u32 = 0x4138AE84;
#[repr(C)]
#[allow(dead_code)]
struct RkvmUserspaceMemoryRegion {
    slot: u32,
    flags: u32,
    guest_phys_addr: u64,
    memory_size: u64,    //bytes
    userspace_addr: u64, //start of the userspace allocated memory
}

static mut GUEST: Option<Ref<GuestWrapper>> = None;
static mut VCPU: Option<Ref<VcpuWrapper>> = None;
impl IoctlHandler for RkvmState {
    type Target<'a> = &'a RkvmState;

    fn pure(_shared: &RkvmState, file: &File, cmd: u32, _arg: usize) -> Result<i32> {
        match cmd {
            IOCTL_KVM_CREATE_VM => {
                if let Err(error) = rkvm_set_vmxon(_shared) {
                    pr_err!("Rkvm: IOCTL_KVM_CREATE_VM failed\n");
                    return Err(error);
                }
                let guest = GuestWrapper::new();

                let guest = match guest {
                    Err(error) => return Err(error),
                    Ok(guest) => guest,
                };
                unsafe {
                    GUEST = Some(guest);
                }
                rkvm_debug!("Rust kvm: IOCTL_KVM_CREATE_VM\n");
                Ok(0)
            }
            IOCTL_KVM_CREATE_VCPU => {
                let guest = unsafe {
                    match &GUEST {
                        Some(e) => e.clone(),
                        None => return Err(Error::ENOENT),
                    }
                };

                rkvm_debug!("Rust kvm: IOCTL_KVM_CREATE_VCPU \n");

                let revision_id = _shared.inner.lock().vmcsconf.revision_id;
                let vcpu0 = VcpuWrapper::new(guest, revision_id);

                let vcpu0 = match vcpu0 {
                    Err(error) => return Err(error),
                    Ok(vcpu0) => vcpu0,
                };

                vcpu0.init(&mut _shared.inner.lock().vmcsconf);
                let va = vcpu0.get_run();
                unsafe {
                    //use for mmap
                    (*file.ptr).private_data = va as *mut c_void;
                    rkvm_debug!("Rust kvm: vcpu create : run = {:x} \n", va);
                    VCPU = Some(vcpu0);
                }

                rkvm_debug!("Rust kvm: IOCTL_KVM_CREATE_VCPU finish\n");

                Ok(0)
            }
            IOCTL_KVM_VCPU_RUN => {
                rkvm_debug!("Rust kvm: IOCTL_KVM_VCPU_RUN\n");
                let vcpu = unsafe {
                    match &VCPU {
                        Some(vcpu) => vcpu,
                        None => return Err(Error::ENOENT),
                    }
                };

                //vcpu.init(&mut _shared.inner.lock().vmcsconf);
                let ret = vcpu.vcpu_run();

                Ok(ret.try_into().unwrap())
            }
            _ => Err(Error::EINVAL),
        }
    }
    fn read(
        _shared: &RkvmState,
        _file: &File,
        cmd: u32,
        writer: &mut UserSlicePtrWriter,
    ) -> Result<i32> {
        match cmd {
            IOCTL_KVM_GET_REGS => {
                rkvm_debug!("Rust kvm: IOCTL_KVM_GET_REGS\n");

                let vcpu = unsafe {
                    match &VCPU {
                        Some(e) => e.clone(),
                        None => return Err(Error::ENOENT),
                    }
                };
                let mut uaddr = RkvmRegs {
                    rax: 0,
                    rbx: 0,
                    rcx: 0,
                    rdx: 0,
                    rsi: 0,
                    rdi: 0,
                    rsp: 0,
                    rbp: 0,
                    r8: 0,
                    r9: 0,
                    r10: 0,
                    r11: 0,
                    r12: 0,
                    r13: 0,
                    r14: 0,
                    r15: 0,
                    rip: 0,
                    rflags: 0,
                };
                vcpu.get_regs(&mut uaddr);
                let len = core::mem::size_of::<RkvmRegs>();
                unsafe {
                    let ptr =
                        core::slice::from_raw_parts((&uaddr as *const RkvmRegs) as *const u8, len);
                    writer.write_raw(ptr.as_ptr(), len)?;
                }
                Ok(0)
            }
            IOCTL_KVM_GET_SREGS => {
                rkvm_debug!("Rust kvm: IOCTL_KVM_GET_SREGS\n");

                let vcpu = unsafe {
                    match &VCPU {
                        Some(e) => e.clone(),
                        None => return Err(Error::ENOENT),
                    }
                };
                let mut uaddr = RkvmSregs::new();
                vcpu.get_sregs(&mut uaddr);
                let len = core::mem::size_of::<RkvmSregs>();
                unsafe {
                    let ptr =
                        core::slice::from_raw_parts((&uaddr as *const RkvmSregs) as *const u8, len);
                    writer.write_raw(ptr.as_ptr(), len)?;
                }
                Ok(0)
            }
            _ => Err(Error::EINVAL),
        }
    }
    fn write(
        _shared: &RkvmState,
        _file: &File,
        cmd: u32,
        reader: &mut UserSlicePtrReader,
    ) -> Result<i32> {
        match cmd {
            IOCTL_KVM_SET_USER_MEMORY_REGION => {
                rkvm_debug!("Rust kvm: IOCTL_KVM_SET_USER_MEMORY_REGION\n");

                let guest = unsafe {
                    match &GUEST {
                        Some(e) => e.clone(),
                        None => return Err(Error::ENOENT),
                    }
                };
                let mut uaddr_ = RkvmUserspaceMemoryRegion {
                    slot: 0,
                    flags: 0,
                    guest_phys_addr: 0,
                    memory_size: 0,
                    userspace_addr: 0,
                };
                let len = core::mem::size_of::<RkvmUserspaceMemoryRegion>();
                rkvm_debug!("Rust kvm: IOCTL_KVM_SET_USER_MEMORY_REGION len={:?}\n", len);

                unsafe {
                    let ptr = core::slice::from_raw_parts_mut(
                        (&mut uaddr_ as *mut RkvmUserspaceMemoryRegion) as *mut u8,
                        len,
                    );
                    reader.read_raw(ptr.as_mut_ptr(), len)?;
                }

                let ret = guest.add_memory_region(
		    uaddr_.slot as u16,
                    uaddr_.userspace_addr,
                    uaddr_.memory_size >> 12,
                    uaddr_.guest_phys_addr,
                );
                ret
            }
            IOCTL_KVM_SET_REGS => {
                rkvm_debug!("Rust kvm: IOCTL_KVM_SET_REGS \n");

                let vcpu = unsafe {
                    match &VCPU {
                        Some(e) => e.clone(),
                        None => return Err(Error::ENOENT),
                    }
                };
                let mut uaddr_ = RkvmRegs {
                    rax: 0,
                    rbx: 0,
                    rcx: 0,
                    rdx: 0,
                    rsi: 0,
                    rdi: 0,
                    rsp: 0,
                    rbp: 0,
                    r8: 0,
                    r9: 0,
                    r10: 0,
                    r11: 0,
                    r12: 0,
                    r13: 0,
                    r14: 0,
                    r15: 0,
                    rip: 0,
                    rflags: 0,
                };

                let len = core::mem::size_of::<RkvmRegs>();
                unsafe {
                    let ptr = core::slice::from_raw_parts_mut(
                        (&mut uaddr_ as *mut RkvmRegs) as *mut u8,
                        len,
                    );
                    reader.read_raw(ptr.as_mut_ptr(), len)?;
                }
                rkvm_debug!(
                    " IOCTL_KVM_SET_REGS: rip={:x}, rax={:x} \n",
                    uaddr_.rip,
                    uaddr_.rax
                );

                vcpu.set_regs(&uaddr_);

                Ok(0)
            }
            IOCTL_KVM_SET_SREGS => {
                rkvm_debug!("Rust kvm: IOCTL_KVM_SET_SREGS \n");

                let vcpu = unsafe {
                    match &VCPU {
                        Some(e) => e.clone(),
                        None => return Err(Error::ENOENT),
                    }
                };
                let mut uaddr_ = RkvmSregs::new();
                let len = core::mem::size_of::<RkvmSregs>();
                unsafe {
                    let ptr = core::slice::from_raw_parts_mut(
                        (&mut uaddr_ as *mut RkvmSregs) as *mut u8,
                        len,
                    );
                    reader.read_raw(ptr.as_mut_ptr(), len)?;
                }

                vcpu.set_sregs(&uaddr_);
                Ok(0)
            }
            _ => Err(Error::EINVAL),
        }
    }
}
