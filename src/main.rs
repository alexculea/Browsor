#![windows_subsystem = "windows"]
#[macro_use]
extern crate simple_error;

mod error;
mod os;
mod ui;

use std::rc::Rc;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::os::sys_browsers;
use crate::os::sys_browsers::Browser;
use crate::ui::{BrowserSelectorUI, ListItem, UserInterface};

fn main() {
    std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicInfo| {
        crate::os::output_panic_text(panic_info.to_string());
        std::process::exit(1);
    }));

    display_prefs();

    let app_name = env!("CARGO_PKG_NAME");
    let app_version = env!("CARGO_PKG_VERSION");
    let target_url = Rc::new(
        std::env::args()
            .nth(1)
            .unwrap_or(String::from("about:home")),
    );

    let mut ui = BrowserSelectorUI::new().expect("Failed to initialize COM or WinUI");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(format!("{} {}", app_name, app_version))
        .with_decorations(true)
        .with_always_on_top(true)
        .with_inner_size(winit::dpi::LogicalSize {
            height: 400 as i16,
            width: 400 as i16,
        })
        .with_resizable(false)
        .with_visible(false)
        .build(&event_loop)
        .expect("Failed to create the main window");
    ui.create(&window)
        .expect("Failed to initialize WinUI XAML.");

    let browsers: Vec<Browser> =
        sys_browsers::read_system_browsers_sync().expect("Could not read browser list");

    let list_items: Vec<ListItem<Browser>> = browsers
        .iter()
        .filter_map(|item| item.try_into().ok())
        .collect();

    ui.set_list(&list_items)
        .expect("Couldn't populate browsers in the UI.");
    ui.set_url(&target_url)
        .expect("Couldn't render URL in the UI.");

    let open_url_clone = Rc::clone(&target_url);
    ui.on_list_item_selected(move |uuid| {
        list_items
            .iter()
            .find(|item| item.uuid == uuid)
            .and_then(|item| Some(item.state.as_ref()))
            .and_then::<std::rc::Rc<Browser>, _>(|browser| {
                os::util::spawn_and_exit(
                    &browser.exe_path,
                    browser.arguments.clone(),
                    &open_url_clone,
                );
                None
            });
    })
    .expect("Cannot set on click event handler.");

    window.set_visible(true);
    event_loop.run(ui::ev_loop::make_ev_loop(target_url, window, ui));
}

fn display_prefs() -> winrt::Result<()> {
  use winrt::ComInterface;
  use bindings::windows::ui::xaml::markup::XamlReader;
  use bindings::windows::ui::xaml::UIElement;
  use crate::ui::win::*;

  mod winapi {
    pub use winapi::shared::windef::{HGDIOBJ, HICON, HWND, POINT};
    pub use winapi::um::wingdi::{DeleteObject, GetBitmapBits, GetObjectW, BITMAP, DIBSECTION};
    pub use winapi::um::winuser::{
        GetCursorPos, GetIconInfo, SetWindowPos, UpdateWindow, ICONINFO, MONITORINFO,
    };
    pub use winapi::ctypes::*;
  }

  mod wrt {
    pub use bindings::windows::ui::xaml::hosting::{
        DesktopWindowXamlSource, IDesktopWindowXamlSourceFactory, WindowsXamlManager,
    };

    pub use bindings::windows::storage::streams::{DataWriter, IBuffer, IDataWriterFactory};

    pub use bindings::windows::foundation::{
        IPropertyValue, IReference, IStringable, PropertyType, PropertyValue,
    };
    pub use bindings::windows::graphics::imaging::{
        BitmapAlphaMode, BitmapPixelFormat, ISoftwareBitmapFactory, SoftwareBitmap,
    };
    pub use bindings::windows::ui::view_management::{UIColorType, UISettings};
    pub use bindings::windows::ui::xaml::controls::{
        Button, ColumnDefinition, Grid, IButtonFactory, IGridFactory, IListBoxFactory,
        IListViewFactory, IRelativePanelFactory, IScrollViewerStatics, IStackPanelFactory, Image,
        ItemClickEventArgs, ItemClickEventHandler, ItemsControl, ListBox, ListView,
        ListViewSelectionMode, Orientation, Panel, RelativePanel, RowDefinition, ScrollMode,
        ScrollViewer, StackPanel, TextBlock,
    };
    pub use bindings::windows::ui::xaml::interop::{TypeKind, TypeName};
    pub use bindings::windows::ui::xaml::media::imaging::{BitmapImage, SoftwareBitmapSource};
    pub use bindings::windows::ui::xaml::media::{ImageSource, SolidColorBrush};
    pub use bindings::windows::ui::xaml::{
        FrameworkElement, GridLength, GridUnitType, RoutedEventHandler, Thickness, UIElement,
        VerticalAlignment,
    };
    pub use bindings::windows::ui::Color;
  }

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new()
      .with_title(String::from("Preferences"))
      .with_decorations(true)
      .with_always_on_top(true)
      .with_inner_size(winit::dpi::LogicalSize {
          height: 400 as i16,
          width: 400 as i16,
      })
      .with_resizable(false)
      .with_visible(false)
      .build(&event_loop)
      .expect("Failed to create the main window");
  
  let mut xaml_isle = init_win_ui_xaml()?;
  let xaml = std::fs::read_to_string("src\\main.xaml").expect("Cant read XAML file");
  let ui_container = XamlReader::load(xaml).expect("Failed loading XAML").query::<UIElement>();
  let size = window.inner_size();
  
  xaml_isle.hwnd = attach_window_to_xaml(&window, &mut xaml_isle)?;
  update_xaml_island_size(&xaml_isle, size)?;
  unsafe {
      winapi::UpdateWindow(xaml_isle.hwnd_parent as winapi::HWND);
  }

  xaml_isle
    .desktop_source
    .set_content(ui_container.to_owned())?;
  
  //let container = ComInterface::query::<wrt::Panel>(&ui_container);
  
  window.set_visible(true);
  event_loop.run(move |event, _, control_flow| {  
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::{ControlFlow};

    *control_flow = ControlFlow::Wait;
    match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => {
            *control_flow = ControlFlow::Exit;
        }
        _ => ()
    }
  });
}