// For clarity purposes keep all WinRT imports under wrt:: 
// winrt is a different crate dealing with types for calling the imported resources
// TODO: Find a better name rather than `wrt` to avoid confusion btw `wrt` and `winrt`
mod wrt {
    pub use bindings::windows::ui::xaml::hosting::{
        DesktopWindowXamlSource, IDesktopWindowXamlSourceFactory, WindowsXamlManager,
    };

    pub use bindings::windows::storage::streams::{
        DataWriter, IDataWriterFactory, IBuffer,
    };

    pub use bindings::windows::foundation::PropertyValue;
    pub use bindings::windows::ui::xaml::controls::{
        Button, IButtonFactory, IListBoxFactory, IListViewFactory, IRelativePanelFactory,
        IStackPanelFactory, ListBox, ListView, ListViewSelectionMode, RelativePanel, StackPanel,
        Orientation,
        TextBlock,
        Image
    };
    pub use bindings::windows::ui::xaml::{RoutedEventHandler, Thickness, UIElement};
    pub use bindings::windows::ui::xaml::media::imaging::{SoftwareBitmapSource, BitmapImage};
    pub use bindings::windows::ui::xaml::media::{ImageSource};
    pub use bindings::windows::graphics::imaging::{
        SoftwareBitmap, ISoftwareBitmapFactory, BitmapPixelFormat, BitmapAlphaMode,
    };
}

use winapi::shared::windef::HWND;
use winapi::um::winuser::{SetWindowPos, UpdateWindow};

use winit::event_loop::EventLoopProxy;

// use crate::initialize_with_window::*;
use crate::desktop_window_xaml_source::IDesktopWindowXamlSourceNative;
use crate::util::{get_hwnd, as_u8_slice};

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

pub struct ListItem {
    pub title: String,
    pub subtitle: String,
}

pub struct UI<'a> {
    pub xaml_isle: &'a XamlIslandWindow,
    pub event_loop: &'a winit::event_loop::EventLoopProxy<BSEvent>,
    pub browser_list: &'a Vec<ListItem>,
    pub url: &'a String,
}

pub fn init_win_ui_xaml() -> winrt::Result<XamlIslandWindow> {
    use winrt::Object;
    let mut xaml_isle = XamlIslandWindow::default();
    xaml_isle.win_xaml_mgr = wrt::WindowsXamlManager::initialize_for_current_thread()?;
    xaml_isle.desktop_source =
        winrt::factory::<wrt::DesktopWindowXamlSource, wrt::IDesktopWindowXamlSourceFactory>()?
            .create_instance(Object::default(), &mut Object::default())?;
    xaml_isle.idesktop_source = xaml_isle.desktop_source.clone().into();

    Ok(xaml_isle)
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

    container.children()?.append(&button)?;
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
    call_to_action_top_row.set_text("You are about to open URL:")?;
    call_to_action_bottom_row.set_text(ui.url as &str)?;
    ui_container.children()?.append(call_to_action_top_row)?;
    ui_container.children()?.append(call_to_action_bottom_row)?;

    let list = create_list(ui.xaml_isle, ui.event_loop, ui.browser_list)?;
    ui_container.children()?.append(list)?;

    Ok(ui_container.into())
}

pub fn create_list_item(title: &str, subtext: &str) -> winrt::Result<wrt::UIElement> {
    let list_item_margins = wrt::Thickness {
        top: 0.,
        left: 15.,
        right: 0.,
        bottom: 0.,
    };
    let root_stack_panel = create_stack_panel()?;
    root_stack_panel.set_orientation(wrt::Orientation::Horizontal)?;

    let image = create_dummy_image()?;
    let name_version_stack_panel = create_stack_panel()?;
    name_version_stack_panel.set_margin(&list_item_margins)?;

    let title_block = wrt::TextBlock::new()?;
    title_block.set_text(title as &str)?;

    let subtitle_block = wrt::TextBlock::new()?;
    subtitle_block.set_text(subtext as &str)?;

    name_version_stack_panel.children()?.append(title_block)?;
    name_version_stack_panel.children()?.append(subtitle_block)?;
    root_stack_panel.children()?.append(image)?;
    root_stack_panel.children()?.append(name_version_stack_panel)?;
    Ok(root_stack_panel.into())
}

pub fn create_stack_panel() -> winrt::Result<wrt::StackPanel> {
    let stack_panel = winrt::factory::<wrt::StackPanel, wrt::IStackPanelFactory>()?
        .create_instance(winrt::Object::default(), &mut winrt::Object::default())?;

    Ok(stack_panel)
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
    list_control.set_selection_mode(wrt::ListViewSelectionMode::Single)?;

    let mut sorted_items = *list.to_owned();
    sorted_items.sort_unstable_by_key(|item| item.title.clone());
    for item in list {
        let item = create_list_item(item.title.as_str(), item.subtitle.as_str())?;
        list_control.items()?.append(winrt::Object::from(item))?;
    }
    list_control.set_selected_index(0)?;

    Ok(list_control.into())
}

pub fn create_dummy_image() -> winrt::Result<wrt::Image> {
    let buffer = [0xFF0000AA; 1024];
    let data_writer = wrt::DataWriter::new()?;
    data_writer.write_bytes(as_u8_slice(&buffer[..]))?;
    
    let i_buffer = data_writer.detach_buffer()?;
    let winrt_bitmap = wrt::SoftwareBitmap::create_copy_with_alpha_from_buffer(
        i_buffer,
        wrt::BitmapPixelFormat::Rgba8,  
        32, 
        32,
        wrt::BitmapAlphaMode::Straight
    )?;

    // ToDO: Can we achieve the same thing without this conversion?
    // Background: ImageSource.SetBitmapAsync will throw an exception if 
    // the bitmap set is not Pixel Format: BGRA8, BitmapAlphaMode: Premulitplied
    // Does it work setting these flags without any pixel conversion?
    let winui_friendly_bmp = wrt::SoftwareBitmap::convert_with_alpha(winrt_bitmap, wrt::BitmapPixelFormat::Bgra8, wrt::BitmapAlphaMode::Premultiplied)?;

    let image_control = wrt::Image::new()?;
    let img_src: wrt::SoftwareBitmapSource = wrt::SoftwareBitmapSource::new()?;
    img_src.set_bitmap_async(winui_friendly_bmp)?;

    image_control.set_source(wrt::ImageSource::from(img_src))?;

    return Ok(image_control);
}


/// Makes a standard RGBA into a single u32 with premultiplied alpha
/// as required by MS XAML
pub fn rgba_to_bgra(r: u8, g: u8, b: u8, a: u8) -> u32 {
    let res_r = ((r as u32) * (a as u32) / 255) as u32;
    let res_g = (g * a / 255) as u32;
    let res_b = (b * a / 255) as u32;

    let b_val = res_b << 24;
    let g_val = res_g << 16;
    let r_val = res_r << 8;

    // might not work with big endian
    b_val | g_val | r_val | a as u32
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
