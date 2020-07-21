/* For clarity purposes keep all WinRT imports under wrt:: */
/* winrt is a different crate dealing with types for calling the imported resources */
mod wrt {
  pub use bindings::windows::ui::xaml::hosting::{
    DesktopWindowXamlSource, IDesktopWindowXamlSourceFactory, WindowsXamlManager,
  };

  pub use bindings::windows::foundation::PropertyValue;
  pub use bindings::windows::ui::xaml::controls::{
    Button, IButtonFactory, IListBoxFactory, IRelativePanelFactory, ListBox, RelativePanel, TextBlock,
  };
  pub use bindings::windows::ui::xaml::{Thickness, RoutedEventHandler};
}

use winapi::shared::windef::HWND;
use winapi::um::winuser::{SetWindowPos, UpdateWindow};

use winit::event_loop::EventLoopProxy;

// use crate::initialize_with_window::*;
use crate::desktop_window_xaml_source::IDesktopWindowXamlSourceNative;
use crate::util::get_hwnd;

pub struct XamlIslandWindow {
  pub hwnd_parent: *mut std::ffi::c_void,

  // the container that draws the DirectComposition stuff to render
  // the modern Windows UI
  pub hwnd: *mut std::ffi::c_void,

  // COM class having the DirectComposition resources
  // has to be initialized first and destroyed last
  pub win_xaml_mgr: wrt::WindowsXamlManager,

  // DesktopWindowXamlSource COM base class
  pub desktop_source: wrt::DesktopWindowXamlSource,

  // IDesktopWindowXamlSource COM derived from DesktopWindowXamlSource above
  // and contains the 'attach' function for using it with existing HWND
  pub idesktop_source: IDesktopWindowXamlSourceNative,
}

impl Default for XamlIslandWindow {
  fn default() -> XamlIslandWindow {
    unsafe {
      XamlIslandWindow {
        hwnd_parent: std::ptr::null_mut(),
        hwnd: std::ptr::null_mut(),
        idesktop_source: std::mem::zeroed(),
        desktop_source: std::mem::zeroed(),
        win_xaml_mgr: std::mem::zeroed(),
      }
    }
  }
}

pub fn init_win_ui_xaml(xaml_isle: &mut XamlIslandWindow) -> winrt::Result<()> {
  use winrt::Object;
  xaml_isle.win_xaml_mgr = wrt::WindowsXamlManager::initialize_for_current_thread()?;
  xaml_isle.desktop_source =
    winrt::factory::<wrt::DesktopWindowXamlSource, wrt::IDesktopWindowXamlSourceFactory>()?
      .create_instance(Object::default(), &mut Object::default())?;
  xaml_isle.idesktop_source = xaml_isle.desktop_source.clone().into();

  Ok(())
}

pub fn attach_window_to_xaml(
  window: &winit::window::Window,
  xaml_isle: &mut XamlIslandWindow,
) -> winrt::Result<*mut std::ffi::c_void> {
  xaml_isle.hwnd_parent = get_hwnd(window) as *mut std::ffi::c_void;

  xaml_isle
    .idesktop_source
    .attach_to_window(xaml_isle.hwnd_parent)?;
  return xaml_isle.idesktop_source.get_window_handle();
}

pub fn update_xaml_island_size(
  xaml_isle: &XamlIslandWindow,
  size: winit::dpi::PhysicalSize<u32>,
) -> winrt::Result<()> {
  unsafe {
    SetWindowPos(
      xaml_isle.hwnd as HWND,
      std::ptr::null_mut(),
      0,
      0,
      size.width as i32,
      size.height as i32,
      0x40,
    );

    UpdateWindow(xaml_isle.hwnd as HWND);
  }

  Ok(())
}

#[derive(Debug)]
pub enum BSEvent {
  BrowserSelected(u32),
  Close,
}

pub fn create_dummy_ui(
  xaml_isle: &XamlIslandWindow,
  ev_loop: winit::event_loop::EventLoopProxy<BSEvent>,
) -> winrt::Result<()> {
  let container = winrt::factory::<wrt::RelativePanel, wrt::IRelativePanelFactory>()?
    .create_instance(winrt::Object::default(), &mut winrt::Object::default())?;
  // let button = Button::new()?;
  let button = winrt::factory::<wrt::Button, wrt::IButtonFactory>()?
    .create_instance(winrt::Object::default(), &mut winrt::Object::default())?;
  let button_text_prop: winrt::Object = wrt::PropertyValue::create_string("Hello world my dear")?;
  button.set_content(button_text_prop)?;
  wrt::RelativePanel::set_align_bottom_with_panel(&button, true)?;
  wrt::RelativePanel::set_align_right_with_panel(&button, true)?;
  button.click(wrt::RoutedEventHandler::new(move |_, _| {
    let _ = ev_loop.send_event(BSEvent::Close);
    Ok(())
  }))?;

  container.children()?.append(&button);
  container.update_layout()?;

  xaml_isle.desktop_source.set_content(container)?;
  Ok(())
}

pub fn create_list(
  xaml: &XamlIslandWindow,
  ev_loop: EventLoopProxy<BSEvent>,
  list: Vec<String>,
) -> winrt::Result<()> {
  let container = winrt::factory::<wrt::RelativePanel, wrt::IRelativePanelFactory>()?
    .create_instance(winrt::Object::default(), &mut winrt::Object::default())?;

  let list_control = winrt::factory::<wrt::ListBox, wrt::IListBoxFactory>()?.create_instance(winrt::Object::default(), &mut winrt::Object::default())?;
  container.set_margin(wrt::Thickness {
    top: 10., left: 10., right: 10., bottom: 10.
  })?;
  // wrt::RelativePanel::set_align_horizontal_center_with_panel(&list_control, true)?;
  // wrt::RelativePanel::set_align_vertical_center_with_panel(&list_control, true)?;

  for browser in list {
    let tb = wrt::TextBlock::new()?;
    let text: &str = &browser;
    tb.set_text(text);
    list_control.items()?.append(winrt::Object::from(tb))?;
  }
  
  container.children()?.append(&list_control);
  container.update_layout()?;
  xaml.desktop_source.set_content(container)?;

  Ok(())
}

//
// These help with creating WinUI dialogs
//
// trait InitializeWithWindow {
//   fn initialize_with_window<O: RuntimeType + ComInterface>(
//       &self,
//       object: &O,
//   ) -> winrt::Result<()>;
// }

// impl<T> InitializeWithWindow for T
// where
//   T: HasRawWindowHandle,
// {
//   fn initialize_with_window<O: RuntimeType + ComInterface>(
//       &self,
//       object: &O,
//   ) -> winrt::Result<()> {
//       // Get the window handle
//       let window_handle = self.raw_window_handle();
//       let window_handle = match window_handle {
//           raw_window_handle::RawWindowHandle::Windows(window_handle) => window_handle.hwnd,
//           _ => panic!("Unsupported platform!"),
//       };

//       let init: InitializeWithWindowInterop = object.try_into()?;
//       init.initialize(window_handle)?;
//       Ok(())
//   }
// }

// eventHandler = move | | -> {
//   use bindings::windows::ui::popups::MessageDialog;
//   let dialog = MessageDialog::create("Test").unwrap();
//   window.initialize_with_window(&dialog).unwrap();
//   dialog.show_async().unwrap();
//   println!("KeyState-{}", input.scancode);
// }
