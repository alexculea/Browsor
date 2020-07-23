use simple_error::{SimpleResult as Result};
mod winapi {
  pub use ::winapi::um::winbase::{GetBinaryTypeW};
  pub use ::winapi::um::winver::{
    GetFileVersionInfoSizeW,
    GetFileVersionInfoW, 
    VerQueryValueW
  };
}


// https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#maximum-path-length-limitation
const WINDOWS_LONG_PATH_PREFIX: &str = r#"\\?\"#;

#[derive(Debug)]
#[derive(Clone)]
#[derive(Default)]
pub struct Browser {
  pub exe_path: String,
  pub name: String,
  pub icon: String,
  pub exe_exists: bool,
  pub icon_exists: bool,
  pub version: VersionInfo,
}

// impl Default for Browser {
//   fn default() -> Browser { 
//     Browser {
//       exe_path: "".into(),
//       name: "".into(),
//       version: "".into(),
//       icon: "".into(),
//     }
//   }
// }

#[derive(Debug)]
#[derive(Clone)]
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


#[derive(Debug)]
#[derive(Clone)]
#[derive(Default)]
pub struct VersionInfo {
  company_name: String,
  file_description: String,
  file_version: String,
  product_version: String,
  binary_type: BinaryType,
}

pub fn read_system_browsers_sync() -> Result<Vec<Browser>> {
  let path32 = "SOFTWARE\\Clients\\StartMenuInternet";
  let path64 = "SOFTWARE\\WOW6432Node\\Clients\\StartMenuInternet";

  let mut list = [
    read_browsers_from_reg_path_sync(path32)?,
    read_browsers_from_reg_path_sync(path64)?,
  ].concat();
  list.dedup_by(|a, b| {a.exe_path == b.exe_path });

  for browser in list.iter_mut() {
    // let version = read_browser_exe_version_info(&[WINDOWS_LONG_PATH_PREFIX, &browser.exe_path].concat());
    match read_browser_exe_version_info(&browser.exe_path) {
      Ok(version) => browser.version = version,
      Err(e) => println!("Error with reading browser info for {}. Reason: {}", browser.exe_path, e),
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

  let browser_root_key = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
    .open_subkey(reg_path)?;

  let browser_name: String = browser_root_key.get_value("")?; // empty gives us (Default)

  let shell_open_command_key = browser_root_key.open_subkey(shell_reg_path)?;
  let mut exe_path: String = shell_open_command_key
    .get_value("")?;
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

pub fn read_browser_exe_version_info(path: &str) -> Result<VersionInfo> {
  const SCS_32BIT_BINARY: u32 = 0;
  const SCS_64BIT_BINARY: u32 = 6;
  const SCS_DOS_BINARY: u32 = 1;
  const SCS_OS216_BINARY: u32 = 5;
  const SCS_WOW_BINARY: u32 = 2;
  
  let mut product_version = String::from("");
  let file_path_wide = crate::util::str_to_wide(path);
  let binary_type = unsafe {
    let mut win_api_binary_type: u32 = 0;
    if winapi::GetBinaryTypeW(file_path_wide.as_ptr(), &mut win_api_binary_type) < 1 {
      bail!("Cannot read binary type with GetBinaryTypeW for file {}", path);
    }

    match win_api_binary_type {
      SCS_32BIT_BINARY | SCS_DOS_BINARY | SCS_WOW_BINARY => BinaryType::Bits32,
      SCS_64BIT_BINARY => BinaryType::Bits64,
      _ => BinaryType::None,
    }
  };

  let mut dword: u32 = 0;
  let file_version_size: u32 = unsafe { winapi::GetFileVersionInfoSizeW(file_path_wide.as_ptr(), &mut dword) };
  if file_version_size == 0 {
    bail!("Cannot read file version size with GetFileVersionInfoSizeExW for {}", path);
  }

  unsafe {
    let mut buffer: Vec<u16> = Vec::with_capacity(file_version_size as usize);
    if winapi::GetFileVersionInfoW(file_path_wide.as_ptr(), 0, file_version_size, buffer.as_mut_ptr() as *mut std::ffi::c_void ) == 0 {
      bail!("Cannot read file version size with GetFileVersionInfoExW for {}", path);
    }

    // pointer holding mem position within buffer above to the requested info
    let mut buffer_pointer: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut size = 0;
    let version_info_prop = crate::util::str_to_wide("\\StringFileInfo\\ProductVersion");
    let result = winapi::VerQueryValueW(buffer.as_mut_ptr() as *mut std::ffi::c_void, version_info_prop.as_ptr(), &mut buffer_pointer, &mut size);
    if result > 0 && size > 0 {
      let mut prd_ver: Vec<u16> = Vec::with_capacity(size as usize);
      std::ptr::copy(buffer_pointer, prd_ver.as_mut_ptr() as *mut std::ffi::c_void, size as usize);
      product_version = crate::util::wide_to_str(&prd_ver);
    }
  }

  Ok(VersionInfo {
    binary_type,
    product_version,
    ..Default::default()
  })
}

pub fn open_url(url: &String, browser: &Browser) {
  println!("URL Open requested with {:?}\nUrl: {}", browser, url);
}
