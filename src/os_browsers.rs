use simple_error::SimpleResult as Result;
mod winapi {
    pub use winapi::shared::minwindef::DWORD;
    pub use winapi::shared::windef::HICON;
    pub use winapi::um::shellapi::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    pub use winapi::um::winbase::GetBinaryTypeW;
    pub use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
    pub use winapi::um::errhandlingapi::GetLastError;
    pub use winapi::um::winnls::GetUserDefaultUILanguage;
}

// https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#maximum-path-length-limitation
const WINDOWS_LONG_PATH_PREFIX: &str = r#"\\?\"#;

#[derive(Debug, Clone)]
pub struct Browser {
    pub exe_path: String,
    pub name: String,
    pub icon: String,
    pub handle_icon: winapi::HICON,
    pub exe_exists: bool,
    pub icon_exists: bool,
    pub version: VersionInfo,
}

impl Default for Browser {
    fn default() -> Browser {
        Browser {
            exe_path: String::default(),
            name: String::default(),
            version: VersionInfo::default(),
            icon: String::default(),
            exe_exists: false,
            icon_exists: false,
            handle_icon: std::ptr::null_mut(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryType {
    Bits32,
    Bits64,
    None,
}

impl Default for BinaryType {
    fn default() -> BinaryType {
        BinaryType::None
    }
}

#[derive(Debug, Clone, Default)]
pub struct VersionInfo {
    company_name: String,
    file_description: String,
    product_version:  String,
    product_name:  String,
    binary_type: BinaryType,
}

pub fn read_system_browsers_sync() -> Result<Vec<Browser>> {
    let path32 = "SOFTWARE\\Clients\\StartMenuInternet";
    let path64 = "SOFTWARE\\WOW6432Node\\Clients\\StartMenuInternet";

    let mut list = [
        read_browsers_from_reg_path_sync(path32)?,
        read_browsers_from_reg_path_sync(path64)?,
    ]
    .concat();
    list.dedup_by(|a, b| a.exe_path == b.exe_path);

    for browser in list.iter_mut() {
        // let version = read_browser_exe_version_info(&[WINDOWS_LONG_PATH_PREFIX, &browser.exe_path].concat());
        match read_browser_exe_info(&browser.exe_path) {
            Ok(version) => browser.version = version,
            Err(e) => println!(
                "Error with reading browser info for {}. Reason: {}",
                browser.exe_path, e
            ),
        }

        match get_exe_file_icon(&browser.exe_path) {
            Ok(icon) => browser.handle_icon = icon,
            Err(e) => println!(
                "Error loading icon from file {}, Reason: {}",
                browser.exe_path, e
            ),
        }
    }
    Ok(list)
}

pub fn read_browsers_from_reg_path_sync(win_reg_path: &str) -> Result<Vec<Browser>> {
    let mut browsers: Vec<Browser> = Vec::new();
    let root = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey(win_reg_path)
        .unwrap();

    for key in root.enum_keys().map(|x| x.unwrap()) {
        match read_browser_info_from_reg_key(&[win_reg_path, "\\", &key].join("")) {
            Ok(browser) => browsers.push(browser),
            Err(e) => println!("Error reading browser info: {:?}", e),
        }
    }
    for (name, value) in root.enum_values().map(|x| x.unwrap()) {
        println!("\t{} = {:?}", name, value);
    }
    Ok(browsers)
}

pub fn read_browser_info_from_reg_key(reg_path: &str) -> std::io::Result<Browser> {
    let shell_reg_path = "shell\\open\\command";
    let icon_reg_path = "DefaultIcon";

    let browser_root_key =
        winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE).open_subkey(reg_path)?;

    let browser_name: String = browser_root_key.get_value("")?; // empty gives us (Default)

    let shell_open_command_key = browser_root_key.open_subkey(shell_reg_path)?;
    let mut exe_path: String = shell_open_command_key.get_value("")?;
    exe_path = exe_path.replace("\"", "");

    let icon_key = browser_root_key.open_subkey(icon_reg_path)?;
    let icon = icon_key.get_value("")?;

    Ok(Browser {
        name: browser_name,
        exe_path: exe_path,
        icon,
        ..Browser::default()
    })
}

pub fn read_browser_exe_info(path: &str) -> Result<VersionInfo> {
    let mut returnValue = read_exe_version_info(path)?;
    returnValue.binary_type = read_exe_arch(path)?;

    Ok(returnValue)
}

/// For the given `path` it returns the architecture of the 
/// executable to be either 32 or 64 bits.
pub fn read_exe_arch(path: &str) -> Result<BinaryType> {
    // WinAPI rust crate is missing the SCS_ constants thus
    // we need to define the values here
    // https://github.com/retep998/winapi-rs/issues/930
    #[repr(u32)]
    #[derive(Clone, Copy)]
    enum WinApiBinaryType {
        None = 12345678,
        Bits32 = 0,
        Bits64 = 6,
        Dos = 1,
        Wow = 2
    }

    let file_path_wide = crate::util::str_to_wide(path);    
    let win_api_binary_type: WinApiBinaryType = WinApiBinaryType::None;
    let api_call_result = unsafe { 
        winapi::GetBinaryTypeW(file_path_wide.as_ptr(), &mut (win_api_binary_type as u32) as *mut u32)
    };

    if api_call_result < 1 {
        bail!(
            "Cannot read binary type with GetBinaryTypeW for file {}",
            path
        );
    }


    Ok(match win_api_binary_type {
        WinApiBinaryType::Bits32 | WinApiBinaryType::Dos | WinApiBinaryType::Wow => BinaryType::Bits32,
        WinApiBinaryType::Bits64 => BinaryType::Bits64,
        _ => BinaryType::None,
    })   
}

/// Reads certain file attributes specific to Windows executables as per the fields
/// in `VersionInfo` struct based on the given `path`
///
/// The fields read are:
/// - ProductName
/// - CompanyName
/// - ProductVersion
///
/// ### Implementation details
/// The implementation is overly complicated due to the goal of having a correct
/// implementation as per Microsoft Docs, thus what we do is:
///  - ask the OS what size is needed to hold the whole blob containing the file version fields
///  - allocate a buffer with that size and ask the OS to copy the hole blob in our buffer
///  - the blob contains some Windows specific hierarchy structures where the data we're interested in is all beneath a certain language code
///  - we ask the OS to tell us what languages are there in the .exe file
///  - we ask the OS what is the OS setting for the user's language and we pick .exe language that matches the UI default or the language neutral entry which Windows defines it as a lang code of 0, or the first element found
///  - we ask the for specific values of the properties `ProductName`, `CompanyName`, `ProductVersion` and if they're not  `UTF-16` we convert them based on the indicated `Code Page`.
pub fn read_exe_version_info(path: &str) -> Result<VersionInfo> {
    const UTF16_WINDOWS_CODE_PAGE: u16 = 1200; 
    let file_path_wide = crate::util::str_to_wide(path);
    let file_version_size: u32 =
        unsafe { winapi::GetFileVersionInfoSizeW(file_path_wide.as_ptr(), &mut 0) };
    if file_version_size == 0 {
        bail!(
            "Cannot read file version size with GetFileVersionInfoSizeExW for {}",
            path
        );
    }

    unsafe {
        let mut version_info_blob: Vec<u8> = Vec::with_capacity(file_version_size as usize);
        if winapi::GetFileVersionInfoW(
            file_path_wide.as_ptr(),
            0,
            file_version_size,
            version_info_blob.as_mut_ptr() as *mut std::ffi::c_void,
        ) == 0
        {
            bail!(
                "Cannot get file version info data with GetFileVersionInfoW for {}",
                path
            );
        }

        #[repr(C)]
        #[derive(Debug)]
        struct LANGANDCODEPAGE {
            wLanguage: u16,
            wCodePage: u16
        };
        type PCLANGANDCODEPAGE = *const LANGANDCODEPAGE;

        // pointer within `buffer` var above based on the sub block given to VerQueryValueW
        let mut out_pointer = std::ptr::null_mut();
        
        // the number of bytes VerQueryValueW has written for the the requested sub block from within the `version_info_blob`
        let mut out_size: u32 = 0;
        
        let translations_sub_block = crate::util::str_to_wide("\\VarFileInfo\\Translation");

        let result = winapi::VerQueryValueW(
            version_info_blob.as_ptr() as *mut std::ffi::c_void,
            translations_sub_block.as_ptr(),
            &mut out_pointer,
            &mut out_size,
        );

        println!("Address of the verinfo buffer: {:p}", &version_info_blob);
        println!("Address of the VerQueryValue pointer: {:p}", out_pointer);
        let raw_buff = std::slice::from_raw_parts::<u8>(out_pointer as *const u8, out_size as usize);
        println!("Raw buffer:\n{:?}", raw_buff);

        if result == 0 || out_size == 0 || out_pointer == std::ptr::null_mut() {
            bail!("Failed to read version info for {}. GetLastError: {:#x}", path, winapi::GetLastError());
        }

        let translations_len = out_size as usize / std::mem::size_of::<LANGANDCODEPAGE>();
        
        // TODO: do we need to forget this because it's technically part of the version_info_blob?
        let translations: &[LANGANDCODEPAGE] = std::slice::from_raw_parts(out_pointer as PCLANGANDCODEPAGE, translations_len);

        let user_lang_id = winapi::GetUserDefaultUILanguage();
        let default_lang_id = 0; // 0 means language neutral

        // look at the translations list and find the one matching
        // the OS language (user_lang_id) or find the language neutral one (default_lang_id)
        // or just return the first element (&translations[0])
        let translation: &LANGANDCODEPAGE = translations
            .iter()
            .find(|item| item.wLanguage == user_lang_id)
            .unwrap_or_else(|| translations
                .iter()
                .find(| item | item.wLanguage == default_lang_id)
                .unwrap_or_else(|| &translations[0])
            );

        let base_block = format!("\\StringFileInfo\\{:04x}{:04x}", translation.wLanguage, translation.wCodePage);
        let product_name_block = base_block.clone() + "\\ProductName";
        let company_name_block = base_block.clone() + "\\CompanyName";
        let product_version_block = base_block.clone() + "\\ProductVersion";

        let mut results = Vec::<String>::with_capacity(3);

        for block in [
            &product_name_block,
            &company_name_block,
            &product_version_block
        ].into_iter() {
            // pointer within `buffer` var above based on the sub block given to VerQueryValueW
            let mut out_pointer = std::ptr::null_mut();
        
            // the number of bytes VerQueryValueW has written for the the requested sub block from within the `version_info_blob`
            let mut out_size: u32 = 0;      
            let result = winapi::VerQueryValueW(
                version_info_blob.as_ptr() as *mut std::ffi::c_void,
                crate::util::str_to_wide(block).as_ptr(),
                &mut out_pointer,
                &mut out_size,
            );

            if result == 0 || out_size == 0 || out_pointer == std::ptr::null_mut() {
                results.push(String::from(""));
                continue;
            }

            let mut raw_wide_string: Vec<u16> = vec!(0);
            
            if translation.wCodePage != UTF16_WINDOWS_CODE_PAGE {
                // TODO: do we need to forget this because it's technically part of the version_info_blob?
                let raw_string = std::slice::from_raw_parts(out_pointer as *const i8, out_size as usize).to_vec();
                raw_wide_string = crate::util::ansi_str_to_wide(&raw_string, translation.wCodePage)?;
            } else {
                raw_wide_string = std::slice::from_raw_parts(out_pointer as *const u16, out_size as usize).to_vec();
            }

            let result_str = crate::util::wide_to_str(&raw_wide_string);       
            results.push(result_str);
        }

        if let [
            product_name, 
            company_name,
            product_version,
        ] = results.as_slice() {
            return Ok(VersionInfo {
                product_name: product_name.into(),
                product_version: product_version.into(),
                company_name: company_name.into(),
                ..Default::default()
            })
        } else {
            bail!("Not all required props were found.");
        }
    }
}

pub fn get_exe_file_icon(path: &str) -> Result<winapi::HICON> {
    let wide_path = crate::util::str_to_wide(&path);
    let mut file_info: winapi::SHFILEINFOW =
        unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    let res = unsafe {
        winapi::SHGetFileInfoW(
            wide_path.as_ptr(),
            0,
            &mut file_info,
            std::mem::size_of_val(&file_info) as u32,
            winapi::SHGFI_ICON | winapi::SHGFI_LARGEICON,
        )
    };

    if res == 0 {
        bail!("Cannot get icon with SHGetFileInfoW for {}", path);
    }

    // Icons given by this function should be destroyed
    // with DestroyIcon. At the same time, according to this
    // https://docs.microsoft.com/en-gb/windows/win32/api/winuser/nf-winuser-loadimagea?redirectedfrom=MSDN#remarks
    // it looks like resources are automatically released
    // when the program ends which is what we need here
    // ToDO: investigate if this causes any memory leaks
    Ok(file_info.hIcon)
}

pub fn open_url(url: &String, browser: &Browser) {
    println!("URL Open requested with {:?}\nUrl: {}", browser, url);
}
