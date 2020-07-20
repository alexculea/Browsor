mod desktop_window_xaml_source;
mod initialize_with_window;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use initialize_with_window::*;
use raw_window_handle::HasRawWindowHandle;
use winrt::*;

struct XamlIslandWindow {
    hwnd_parent: *mut std::ffi::c_void,

    // the container that draws the DirectComposition stuff to render
    // the modern Windows UI
    hwnd: *mut std::ffi::c_void,

    // COM class having the DirectComposition resources
    // has to be initialized first and destroyed last
    win_xaml_mgr: bindings::windows::ui::xaml::hosting::WindowsXamlManager,

    // DesktopWindowXamlSource COM base class
    desktop_source: bindings::windows::ui::xaml::hosting::DesktopWindowXamlSource,

    // IDesktopWindowXamlSource COM derived from DesktopWindowXamlSource above
    // and contains the 'attach' function for using it with existing HWND
    idesktop_source: desktop_window_xaml_source::IDesktopWindowXamlSourceNative,
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

unsafe fn initialize_runtime_com() -> winrt::Result<()> {
    use winapi::shared::winerror::S_OK;
    use winapi::winrt::roapi::RoInitialize;

    let result = winrt::ErrorCode::from(Ok(RoInitialize(
        winapi::winrt::roapi::RO_INIT_SINGLETHREADED,
    )));
    if (result.is_ok()) {
        return winrt::Result::Ok(());
    }

    return Err(winrt::Error::from(result));
}

fn create_xaml_island(xaml_isle: &mut XamlIslandWindow) -> winrt::Result<()> {
    use bindings::windows::ui::xaml::hosting::{
        DesktopWindowXamlSource, IDesktopWindowXamlSourceFactory, WindowsXamlManager,
    };
    use core::ptr;
    use desktop_window_xaml_source::IDesktopWindowXamlSourceNative;
    use winrt::Object;
    xaml_isle.win_xaml_mgr = WindowsXamlManager::initialize_for_current_thread()?;
    xaml_isle.desktop_source =
        winrt::factory::<DesktopWindowXamlSource, IDesktopWindowXamlSourceFactory>()?
            .create_instance(Object::default(), &mut Object::default())?;
    xaml_isle.idesktop_source = xaml_isle.desktop_source.clone().into();

    Ok(())
}


fn get_hwnd(window: &winit::window::Window) -> winapi::shared::windef::HWND {
    match window.raw_window_handle() {
        raw_window_handle::RawWindowHandle::Windows(wnd_handle) => wnd_handle.hwnd as winapi::shared::windef::HWND,
        _ => panic!("No MSFT Windows specific window handle. Wrong platform?")
    }
}

fn attach_window_to_xaml(
    window: &winit::window::Window,
    xaml_isle: &mut XamlIslandWindow,
) -> winrt::Result<*mut std::ffi::c_void> {
    xaml_isle.hwnd_parent = get_hwnd(window) as *mut std::ffi::c_void;

    xaml_isle
        .idesktop_source
        .attach_to_window(xaml_isle.hwnd_parent)?;
    return xaml_isle.idesktop_source.get_window_handle();
}

#[derive(Debug)]
enum BSEvent {
    DummyClick,
}

fn create_dummy_ui(
    xaml_isle: &XamlIslandWindow,
    ev_loop: winit::event_loop::EventLoopProxy<BSEvent>,
) -> Result<()> {
    use bindings::windows::foundation::PropertyValue;
    use bindings::windows::ui::xaml::controls::{
        Button, IButtonFactory, IRelativePanelFactory, RelativePanel,
    };
    use bindings::windows::ui::xaml::RoutedEventHandler;
    use winrt::Object;

    let container = winrt::factory::<RelativePanel, IRelativePanelFactory>()?
        .create_instance(Object::default(), &mut Object::default())?;
    // let button = Button::new()?;
    let button = winrt::factory::<Button, IButtonFactory>()?
        .create_instance(Object::default(), &mut Object::default())?;
    let button_text_prop: Object = PropertyValue::create_string("Hello world my dear")?;
    button.set_content(button_text_prop)?;
    RelativePanel::set_align_bottom_with_panel(&button, true)?;
    RelativePanel::set_align_right_with_panel(&button, true)?;
    button.click(RoutedEventHandler::new(move |_, _| {
        let _ = ev_loop.send_event(BSEvent::DummyClick);
        Ok(())
    }))?;

    container.children()?.append(&button);
    container.update_layout()?;

    xaml_isle
        .desktop_source
        .set_content(container)?;
    Ok(())
}

fn update_xaml_island_size(xaml_isle: &XamlIslandWindow, size: winit::dpi::PhysicalSize<u32>) -> Result<()> {
    unsafe {
        winapi::um::winuser::SetWindowPos(
            xaml_isle.hwnd as winapi::shared::windef::HWND,
            std::ptr::null_mut(),
            0,
            0,
            size.width as i32,
            size.height as i32,
            0x40,
        );

        winapi::um::winuser::UpdateWindow(xaml_isle.hwnd as winapi::shared::windef::HWND);
    }

    Ok(())
}

fn hide_window(window: &winit::window::Window) {
    unsafe { 
        winapi::um::winuser::ShowWindow(
            get_hwnd(window), 
            winapi::um::winuser::SW_HIDE
        );
    }
}

fn main() {
    unsafe {
        initialize_runtime_com().expect("Failed to to initialize COM runtime.");
    }
    let mut xaml_isle = XamlIslandWindow::default();
    create_xaml_island(&mut xaml_isle)
        .expect("Failed to initialize WindowsXamlManager or DesktopWindowXamlSource.");

    let event_loop = EventLoop::with_user_event();
    let event_loop_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_decorations(false)
        .build(&event_loop)
        .unwrap();

    xaml_isle.hwnd = attach_window_to_xaml(&window, &mut xaml_isle)
        .expect("Failed to create WinUI host control (HWND).");

    let size = window.inner_size();
    update_xaml_island_size(&xaml_isle, size);

    unsafe {
        winapi::um::winuser::UpdateWindow(xaml_isle.hwnd_parent as winapi::shared::windef::HWND);
    }

    create_dummy_ui(&xaml_isle, event_loop_proxy);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => { 
                hide_window(&window);
                *control_flow = ControlFlow::Exit 
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(_size),
                ..
            } => { update_xaml_island_size(&xaml_isle, _size); },
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } if input.state == winit::event::ElementState::Pressed => {
                use bindings::windows::ui::popups::MessageDialog;
                let dialog = MessageDialog::create("Test").unwrap();
                window.initialize_with_window(&dialog).unwrap();
                dialog.show_async().unwrap();
                println!("KeyState-{}", input.scancode);
            }
            Event::UserEvent(BSEvent::DummyClick) => {
                *control_flow = ControlFlow::Exit;
            },
            _ => (),
        }
    });

    Ok(())
}

trait InitializeWithWindow {
    fn initialize_with_window<O: RuntimeType + ComInterface>(
        &self,
        object: &O,
    ) -> winrt::Result<()>;
}

impl<T> InitializeWithWindow for T
where
    T: HasRawWindowHandle,
{
    fn initialize_with_window<O: RuntimeType + ComInterface>(
        &self,
        object: &O,
    ) -> winrt::Result<()> {
        // Get the window handle
        let window_handle = self.raw_window_handle();
        let window_handle = match window_handle {
            raw_window_handle::RawWindowHandle::Windows(window_handle) => window_handle.hwnd,
            _ => panic!("Unsupported platform!"),
        };

        let init: InitializeWithWindowInterop = object.try_into()?;
        init.initialize(window_handle)?;
        Ok(())
    }
}
