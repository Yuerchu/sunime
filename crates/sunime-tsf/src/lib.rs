mod text_service;

use std::sync::atomic::{AtomicPtr, Ordering};

use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows_core::*;

use text_service::TextService;

static HINSTANCE: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(std::ptr::null_mut());

pub const CLSID_SUNIME: GUID = GUID::from_u128(0x7A3B9C1E_4D5F_6A8B_9C2D_E1F0A3B5C7D9);

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(hinstance: HMODULE, reason: u32, _reserved: *mut core::ffi::c_void) -> BOOL {
    if reason == 1 {
        HINSTANCE.store(hinstance.0, Ordering::Relaxed);
    }
    TRUE
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_INVALIDARG;
    }
    unsafe { *ppv = std::ptr::null_mut() };

    let rclsid = unsafe { &*rclsid };
    if *rclsid != CLSID_SUNIME {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    let factory = ClassFactory;
    let unknown: IUnknown = factory.into();
    let riid = unsafe { &*riid };
    unsafe { unknown.query(riid, ppv) }
}

#[unsafe(no_mangle)]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[implement(IClassFactory)]
struct ClassFactory;

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(&self, _outer: Ref<'_, IUnknown>, riid: *const GUID, ppv: *mut *mut core::ffi::c_void) -> Result<()> {
        let service = TextService::new();
        let unknown: IUnknown = service.into();
        unsafe { unknown.query(riid, ppv).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}
