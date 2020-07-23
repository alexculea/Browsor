/* For clarity purposes keep all WinRT imports under wrt:: */
/* winrt is a different crate dealing with types for calling the imported resources */
mod wrt {
  pub use bindings::windows::ui::xaml::hosting::{
    DesktopWindowXamlSource, IDesktopWindowXamlSourceFactory, WindowsXamlManager,
  };

  pub use bindings::windows::foundation::PropertyValue;
  pub use bindings::windows::ui::xaml::controls::{
    Button, IButtonFactory, IListBoxFactory, IListViewFactory, IRelativePanelFactory,
    IStackPanelFactory, ListBox, ListView, ListViewSelectionMode, RelativePanel, StackPanel,
    TextBlock,
  };
  pub use bindings::windows::ui::xaml::{RoutedEventHandler, Thickness, UIElement};
}

use winapi::shared::windef::HWND;
use winapi::um::winuser::{SetWindowPos, UpdateWindow};

use winit::event_loop::EventLoopProxy;

// use crate::initialize_with_window::*;
use crate::desktop_window_xaml_source::IDesktopWindowXamlSourceNative;
use crate::util::get_hwnd;

#[derive(Debug)]
pub enum BSEvent {
  BrowserSelected(u32),
  Close,
}

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

pub struct ListItem<'a> {
  pub title: &'a str,
  pub subtitle: &'a str,
}

pub struct UI<'a> {
  pub xaml_isle: &'a XamlIslandWindow,
  pub event_loop: &'a winit::event_loop::EventLoopProxy<BSEvent>,
  pub browser_list: &'a Vec<ListItem<'a>>,
  pub url: &'a String,
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

pub fn create_ui(ui: &UI) -> winrt::Result<wrt::UIElement> {
  let ui_container = winrt::factory::<wrt::StackPanel, wrt::IStackPanelFactory>()?
    .create_instance(winrt::Object::default(), &mut winrt::Object::default())?;
  ui_container.set_margin(wrt::Thickness {
    top: 15.,
    left: 15.,
    right: 15.,
    bottom: 15.,
  })?;

  let call_to_action_top_row = wrt::TextBlock::new()?;
  let call_to_action_bottom_row = wrt::TextBlock::new()?;
  call_to_action_top_row.set_text("You are about to open URL:");
  call_to_action_bottom_row.set_text(ui.url as &str);
  ui_container.children()?.append(call_to_action_top_row);
  ui_container.children()?.append(call_to_action_bottom_row);

  let list = create_list(ui.xaml_isle, ui.event_loop, ui.browser_list)?;
  ui_container.children()?.append(list);

  Ok(ui_container.into())
}

pub fn create_list_item(title: &str, subtext: &str) -> winrt::Result<wrt::UIElement> {
  let list_item_margins = wrt::Thickness {
    top: 15.,
    left: 15.,
    right: 15.,
    bottom: 15.,
  };
  let stack_panel = winrt::factory::<wrt::StackPanel, wrt::IStackPanelFactory>()?
    .create_instance(winrt::Object::default(), &mut winrt::Object::default())?;
  stack_panel.set_margin(&list_item_margins);

  let title_block = wrt::TextBlock::new()?;
  title_block.set_text(title as &str);

  let subtitle_block = wrt::TextBlock::new()?;
  subtitle_block.set_text(subtext as &str)?;

  stack_panel.children()?.append(title_block);
  stack_panel.children()?.append(subtitle_block);
  Ok(stack_panel.into())
}

pub fn create_list(
  xaml: &XamlIslandWindow,
  ev_loop: &EventLoopProxy<BSEvent>,
  list: &Vec<ListItem>,
) -> winrt::Result<wrt::UIElement> {
  struct ListPos {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
  };
  let list_pos = ListPos {
    x: 0.,
    y: 0.,
    width: 500.,
    height: 200.,
  };

  let list_control = winrt::factory::<wrt::ListView, wrt::IListViewFactory>()?
    .create_instance(winrt::Object::default(), &mut winrt::Object::default())?;
  list_control.set_margin(wrt::Thickness {
    top: 15.,
    left: 0.,
    right: 0.,
    bottom: 0.,
  })?;
  list_control.set_selection_mode(wrt::ListViewSelectionMode::Single);

  for item in list {
    let item = create_list_item(item.title, item.subtitle)?;
    list_control.items()?.append(winrt::Object::from(item))?;
  }
  list_control.set_selected_index(0);

  Ok(list_control.into())
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
