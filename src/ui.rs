use std::mem::MaybeUninit;
use std::convert::TryInto;

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
        ScrollViewer, ScrollMode, IScrollViewerStatics,
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

mod winapi {
    pub use winapi::shared::windef::{
        HWND,
        HICON,
        HGDIOBJ
    };
    pub use winapi::um::winuser::{
        GetIconInfo,
        SetWindowPos,
        UpdateWindow,
        ICONINFO,
    };
    pub use winapi::um::wingdi::{
        DeleteObject,
        GetObjectW,
        GetBitmapBits,
        DIBSECTION,
        BITMAP,
    };
}


use winit::event_loop::EventLoopProxy;

// use crate::initialize_with_window::*;
use crate::desktop_window_xaml_source::IDesktopWindowXamlSourceNative;
use crate::util::{get_hwnd, as_u8_slice};
use crate::error::{BSResult, BSError};

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

#[derive(Clone)]
pub struct ListItem {
    pub title: String,
    pub subtitle: String,
    pub image: wrt::Image,
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
        winapi::SetWindowPos(
            xaml_isle.hwnd as winapi::HWND,
            std::ptr::null_mut(),
            0,
            0,
            size.width as i32,
            size.height as i32,
            0x40,
        );

        winapi::UpdateWindow(xaml_isle.hwnd as winapi::HWND);
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

pub fn create_list_item(title: &str, subtext: &str, image: wrt::Image) -> winrt::Result<wrt::UIElement> {
    let list_item_margins = wrt::Thickness {
        top: 0.,
        left: 15.,
        right: 0.,
        bottom: 0.,
    };
    let root_stack_panel = create_stack_panel()?;
    root_stack_panel.set_orientation(wrt::Orientation::Horizontal)?;

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
    list: &[ListItem],
) -> winrt::Result<wrt::UIElement> {
    let list_control = winrt::factory::<wrt::ListView, wrt::IListViewFactory>()?
        .create_instance(winrt::Object::default(), &mut winrt::Object::default())?;
    list_control.set_margin(wrt::Thickness {
        top: 15.,
        left: 0.,
        right: 0.,
        bottom: 0.,
    })?;
    list_control.set_selection_mode(wrt::ListViewSelectionMode::Single)?;
    
    // TODO: Fix scroll bars not coming in the list when its height is bigger
    // than the parent. StackPanel parent might have something to do with this:
    // https://stackoverflow.com/questions/41140287/horizontal-scroll-for-stackpanel-doesnt-work/41140885#41140885
    let scroll_viewer_statics = winrt::factory::<wrt::ScrollViewer, wrt::IScrollViewerStatics>()?;
    scroll_viewer_statics.set_vertical_scroll_mode(&list_control, wrt::ScrollMode::Enabled)?;

    let mut sorted_items = list.to_vec();
    sorted_items.sort_unstable_by_key(|item| item.title.clone());
    for item in sorted_items {
        let item = create_list_item(
            item.title.as_str(),
            item.subtitle.as_str(),
            item.image,
        )?;
        list_control.items()?.append(winrt::Object::from(item))?;
    }
    list_control.set_selected_index(0)?;

    Ok(list_control.into())
}

/// From the given WinRT SoftwareBitmap it returns
/// the corresponding WinUI Image XAML control that can be inserted
/// as a node in any UIElement derived object 
pub fn software_bitmap_to_xaml_image(bmp: wrt::SoftwareBitmap) -> winrt::Result<wrt::Image> {
    // ToDO: Can we achieve the same thing without this conversion?
    // Background: ImageSource.SetBitmapAsync will throw an exception if 
    // the bitmap set is not Pixel Format: BGRA8, BitmapAlphaMode: Premulitplied
    // Does it work setting these flags without any pixel conversion?
    let bgra8_bmp = match bmp.bitmap_pixel_format()? {
        wrt::BitmapPixelFormat::Bgra8 => { 
            wrt::SoftwareBitmap::convert_with_alpha(
                bmp,
                wrt::BitmapPixelFormat::Bgra8,
                wrt::BitmapAlphaMode::Premultiplied
            )?
        },
        _ => bmp,
    };

    let image_control = wrt::Image::new()?;
    let img_src: wrt::SoftwareBitmapSource = wrt::SoftwareBitmapSource::new()?;
    img_src.set_bitmap_async(bgra8_bmp)?;
    image_control.set_source(wrt::ImageSource::from(img_src))?;

    return Ok(image_control);
}

pub fn hicon_to_software_bitmap(hicon: winapi::HICON) -> BSResult<wrt::SoftwareBitmap> {
    // TODO: there exists a .net function called 
    let mut icon_info: winapi::ICONINFO = unsafe { MaybeUninit::uninit().assume_init() };
    let icon_result = unsafe{ winapi::GetIconInfo(hicon, &mut icon_info) };
    if icon_result == 0 {
        bail!("Couldn't get icon info for HICON {:?}", hicon);
    }

    let dib_struct_size = std::mem::size_of::<winapi::DIBSECTION>()
        .try_into()
        .unwrap_or(0);
    let bitmap_struct_size = std::mem::size_of::<winapi::BITMAP>()
        .try_into()
        .unwrap_or(0);
    

    let mut dib: winapi::DIBSECTION = unsafe { MaybeUninit::uninit().assume_init() };
    let bytes_read = unsafe { winapi::GetObjectW(
        icon_info.hbmColor as *mut _ as *mut std::ffi::c_void,
        dib_struct_size,
        &mut dib as *mut _ as *mut std::ffi::c_void
    ) };

    if bytes_read == 0 {
        unsafe {
            winapi::DeleteObject(icon_info.hbmColor as winapi::HGDIOBJ);
            winapi::DeleteObject(icon_info.hbmMask as winapi::HGDIOBJ);
        }

        bail!("Error: winapi::GetObject returned 0 on ICONINFO.hbmColor bitmap.");
    }

    // BITMAP size is 32 bytes
    // DIBSECTION is 104 bytes
    let bmp_size_in_bytes 
        = (dib.dsBm.bmHeight * dib.dsBm.bmWidth) * (dib.dsBm.bmBitsPixel as i32 / 8);

    let pixel_bytes_result = match bytes_read {
        bytes_read if bytes_read == bitmap_struct_size => {
            // when GetObject returns the size of the BITMAP structure
            // then dib.dsBm is a device dependent bitmap we need to use GetBitmapBits
            let mut img_bytes = Vec::<u8>::new();
            img_bytes.resize(bmp_size_in_bytes as usize, 0);

            let img_bytes_read = unsafe { 
                winapi::GetBitmapBits(
                    icon_info.hbmColor,
                    bmp_size_in_bytes,
                    img_bytes.as_mut_slice().as_mut_ptr() as *mut std::ffi::c_void
                )
            };
            // TODO: Replace GetBitmapBits with GetDibBits because GetBitmapBits is deprecated

            if img_bytes_read == 0 { 
                Err("winapi::GetBitmapBits read 0 bytes from the ICONINFO.hbmColor") 
            } else { 
                Ok(img_bytes) 
            }
        },
        bytes_read if bytes_read == dib_struct_size => {
            if dib.dsBm.bmBits as usize != 0 {
                Ok(unsafe { 
                    std::slice::from_raw_parts::<u8>(
                        dib.dsBm.bmBits as *const u8,
                        bmp_size_in_bytes as usize
                    ).to_vec()
                })
            } else {
                Err("Unexpected NULL pointer for image bits from DIBSECTION.dsBm.bmBits")
            }
        },
        0 => Err("winapi::GetObject returned 0 on ICONINFO.hbmColor bitmap."),
        _ => Err(
            "Unexpected response from winapi::GetObject, was expecting read bytes \
            to match either the BITMAP struct size or the DIBSECTION struct size."
        ),
    };

    let pixel_bytes = match pixel_bytes_result {
        Ok(bytes) => bytes,
        Err(error) => unsafe { 
            winapi::DeleteObject(icon_info.hbmColor as winapi::HGDIOBJ);
            winapi::DeleteObject(icon_info.hbmMask as winapi::HGDIOBJ);
            bail!(error);
        }
    };

    let raw_pixels = pixel_bytes.chunks_exact(4)
        .map(|chunk| { 
            u32::from_ne_bytes(
                chunk.try_into().expect("Expected chunk size to be 4 bytes when converting to u32")
            ) 
        })
        .collect::<Vec<u32>>();

    let data_writer = wrt::DataWriter::new()?;
    data_writer.write_bytes(as_u8_slice(&raw_pixels[..]))?;
    
    let i_buffer = data_writer.detach_buffer()?;
    let software_bitmap = wrt::SoftwareBitmap::create_copy_with_alpha_from_buffer(
        i_buffer,
        wrt::BitmapPixelFormat::Bgra8,
        dib.dsBm.bmWidth, 
        dib.dsBm.bmHeight,
        wrt::BitmapAlphaMode::Straight
    )?;
    // About the BitmapPixelFormat::Bgra8:
    // Hard coding pixel format to BGRA with 1 byte per color seems to work but it should be
    // detected since there are guarantees the Windows API will always return this format


    unsafe {
        winapi::DeleteObject(icon_info.hbmColor as winapi::HGDIOBJ);
        winapi::DeleteObject(icon_info.hbmMask as winapi::HGDIOBJ);
    }

    return Ok(software_bitmap);
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
