use simple_error::SimpleResult as Result;
use raw_window_handle::HasRawWindowHandle;
use winapi::winrt::roapi::RoInitialize;
use winapi::um::stringapiset::MultiByteToWideChar;

pub unsafe fn initialize_runtime_com() -> winrt::Result<()> {
  let result = winrt::ErrorCode::from(Ok(RoInitialize(
    winapi::winrt::roapi::RO_INIT_SINGLETHREADED, // TODO: Investigate if we need multithreaded due to winnit event loop
  )));

  if result.is_ok() {
    return winrt::Result::Ok(());
  }

  winapi::um::combaseapi::CoInitializeEx(std::ptr::null_mut(), 0x2);

  return Err(winrt::Error::from(result));
}

pub fn get_hwnd(window: &winit::window::Window) -> winapi::shared::windef::HWND {
  match window.raw_window_handle() {
    raw_window_handle::RawWindowHandle::Windows(wnd_handle) => {
      wnd_handle.hwnd as winapi::shared::windef::HWND
    }
    _ => panic!("No MS Windows specific window handle. Wrong platform?"),
  }
}

pub fn hide_window(window: &winit::window::Window) {
  unsafe {
    winapi::um::winuser::ShowWindow(get_hwnd(window), winapi::um::winuser::SW_HIDE);
  }
}

pub fn str_to_wide(string: &str) -> Vec<u16> {
  use std::ffi::OsStr;
  use std::os::windows::ffi::OsStrExt;
  use std::iter::once;

  OsStr::new(string).encode_wide().chain(once(0)).collect()
}

pub fn wide_to_str(buf: &Vec<u16>) -> String {
  String::from_utf16_lossy(&buf)
}

/// From the given buffer `src_string` use the Windows API to convert the
/// ANSI string with the given `code_page`.
///
/// [MSDN Info](https://docs.microsoft.com/en-us/windows/win32/intl/code-pages)
pub fn ansi_str_to_wide(src_string: &Vec<i8>, code_page: u16) -> Result<Vec<u16>> {
  let receiving_capacity = src_string.len() * 2;
  // generally single byte strings use 1 byte per character
  // we allocate twice of that hoping we won't get truncated

  let mut dst_string = Vec::<u16>::with_capacity(receiving_capacity);
  let result = unsafe { MultiByteToWideChar(
    code_page as u32, 
    0, 
    src_string.as_ptr() as *const i8,
    src_string.len() as i32, // size in bytes is the same as the length
    dst_string.as_mut_ptr(), 
    (receiving_capacity - 1) as i32
  ) };
  // this call is safe as long as we trust WinAPI
  // to respect the indicated capacities in cbMultiByte (param 4)
  // and in cchWideChar (param 6)

  if result == 0 {
    bail!("Could not convert the given string. Call GetLastError from WinAPI to find out why.");
  }

  Ok(dst_string)
}

pub fn as_u8_slice(v: &[u32]) -> &[u8] {
  let element_size = std::mem::size_of::<u32>();
  unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * element_size) }
}

pub fn get_exe_file_icon(path: &str) -> Result<winapi::shared::windef::HICON> {
  use winapi::um::shellapi::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};

  let wide_path = crate::os_util::str_to_wide(&path);
  let mut file_info: SHFILEINFOW =
      unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
  let res = unsafe {
      SHGetFileInfoW(
          wide_path.as_ptr(),
          0,
          &mut file_info,
          std::mem::size_of_val(&file_info) as u32,
          SHGFI_ICON | SHGFI_LARGEICON,
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