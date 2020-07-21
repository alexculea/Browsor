use raw_window_handle::HasRawWindowHandle;
use winapi::winrt::roapi::RoInitialize;

pub unsafe fn initialize_runtime_com() -> winrt::Result<()> {
  let result = winrt::ErrorCode::from(Ok(RoInitialize(
    winapi::winrt::roapi::RO_INIT_SINGLETHREADED,
  )));

  if result.is_ok() {
    return winrt::Result::Ok(());
  }

  return Err(winrt::Error::from(result));
}

pub fn get_hwnd(window: &winit::window::Window) -> winapi::shared::windef::HWND {
  match window.raw_window_handle() {
    raw_window_handle::RawWindowHandle::Windows(wnd_handle) => {
      wnd_handle.hwnd as winapi::shared::windef::HWND
    }
    _ => panic!("No MSFT Windows specific window handle. Wrong platform?"),
  }
}

pub fn hide_window(window: &winit::window::Window) {
  unsafe {
    winapi::um::winuser::ShowWindow(get_hwnd(window), winapi::um::winuser::SW_HIDE);
  }
}
