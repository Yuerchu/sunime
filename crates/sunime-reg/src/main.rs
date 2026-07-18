use std::{env, path::PathBuf, process};

use windows::Win32::Globalization::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::TextServices::*;
use windows_core::*;
use windows_registry::LOCAL_MACHINE;

const CLSID_SUNIME: GUID = GUID::from_u128(0x7A3B9C1E_4D5F_6A8B_9C2D_E1F0A3B5C7D9);
const PROFILE_ZH_CN: GUID = GUID::from_u128(0xB2C4D5E6_7F8A_9B0C_1D2E_3F4A5B6C7D8E);

const CATEGORIES: [GUID; 5] = [
    GUID_TFCAT_TIP_KEYBOARD,
    GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
    GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
    GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
    GUID_TFCAT_TIPCAP_COMLESS,
];

fn clsid_string() -> String {
    format!(
        "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
        CLSID_SUNIME.data1, CLSID_SUNIME.data2, CLSID_SUNIME.data3,
        CLSID_SUNIME.data4[0], CLSID_SUNIME.data4[1],
        CLSID_SUNIME.data4[2], CLSID_SUNIME.data4[3],
        CLSID_SUNIME.data4[4], CLSID_SUNIME.data4[5],
        CLSID_SUNIME.data4[6], CLSID_SUNIME.data4[7],
    )
}

fn register_com(dll_path: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let clsid = clsid_string();
    let key_path = format!("SOFTWARE\\Classes\\CLSID\\{clsid}\\InprocServer32");
    let key = LOCAL_MACHINE.create(&key_path)?;
    key.set_string("", dll_path)?;
    key.set_string("ThreadingModel", "Apartment")?;
    println!("  COM CLSID registered: {clsid}");
    Ok(())
}

fn unregister_com() {
    let clsid = clsid_string();
    let _ = LOCAL_MACHINE.remove_tree(&format!("SOFTWARE\\Classes\\CLSID\\{clsid}"));
    println!("  COM CLSID removed");
}

fn register_tsf() -> Result<()> {
    unsafe {
        let profile_mgr: ITfInputProcessorProfileMgr =
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;

        let mut lcid = LocaleNameToLCID(w!("zh-CN"), 0);
        if matches!(lcid, 0 | 0x0C00 | 0x1000) {
            lcid = 0x804;
        }

        profile_mgr.RegisterProfile(
            &CLSID_SUNIME,
            lcid as u16,
            &PROFILE_ZH_CN,
            w!("隼输入法 SunIME").as_wide(),
            &[],
            0,
            HKL::default(),
            0,
            false,
            0,
        )?;
        println!("  TSF profile registered (zh-CN, lcid={lcid:#x})");

        let cat_mgr: ITfCategoryMgr =
            CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        for cat in &CATEGORIES {
            cat_mgr.RegisterCategory(&CLSID_SUNIME, cat, &CLSID_SUNIME)?;
        }
        println!("  TSF categories registered ({})", CATEGORIES.len());
    }
    Ok(())
}

fn unregister_tsf() {
    unsafe {
        if let Ok(cat_mgr) = CoCreateInstance::<_, ITfCategoryMgr>(
            &CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER,
        ) {
            for cat in &CATEGORIES {
                let _ = cat_mgr.UnregisterCategory(&CLSID_SUNIME, cat, &CLSID_SUNIME);
            }
        }
        println!("  TSF categories removed");

        if let Ok(profile_mgr) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
            &CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER,
        ) {
            let mut lcid = LocaleNameToLCID(w!("zh-CN"), 0);
            if matches!(lcid, 0 | 0x0C00 | 0x1000) {
                lcid = 0x804;
            }
            let _ = profile_mgr.UnregisterProfile(&CLSID_SUNIME, lcid as u16, &PROFILE_ZH_CN, 0);
        }
        println!("  TSF profile removed");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let dll_path = args.get(2).map(|s| s.to_string()).unwrap_or_else(|| {
        exe_dir.join("sunime_tsf.dll").to_string_lossy().to_string()
    });

    match args.get(1).map(|s| s.as_str()) {
        Some("--register") | Some("-r") => {
            println!("Registering SunIME...");
            println!("  DLL: {dll_path}");

            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap() };

            if let Err(e) = register_com(&dll_path) {
                eprintln!("  COM failed: {e}");
                eprintln!("  (requires Administrator)");
                process::exit(1);
            }
            if let Err(e) = register_tsf() {
                eprintln!("  TSF failed: {e}");
                process::exit(1);
            }
            println!("Done.");

            unsafe { CoUninitialize() };
        }
        Some("--unregister") | Some("-u") => {
            println!("Unregistering SunIME...");
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap() };
            unregister_tsf();
            unregister_com();
            println!("Done.");
            unsafe { CoUninitialize() };
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  sunime-reg --register [dll_path]   Register (requires Admin)");
            eprintln!("  sunime-reg --unregister            Remove");
        }
    }
}
