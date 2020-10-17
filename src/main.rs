#[macro_use]
extern crate simple_error;

mod desktop_window_xaml_source;
mod os_browsers;
mod ui;
mod util;
mod error;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    unsafe {
        util::initialize_runtime_com().expect("Failed to to initialize COM runtime.");
    }

    let url: String = "http://www.google.com".into();

    // Initialize WinUI XAML before creating the winit EventLoop
    // or winit throws: thread 'main'
    // panicked at 'either event handler is re-entrant (likely), or no event
    // handler is registered (very unlikely)'
    let mut xaml_isle = ui::init_win_ui_xaml().expect("Failed to initialize WinUI XAML.");

    let event_loop = EventLoop::with_user_event();
    let event_loop_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    xaml_isle.hwnd = ui::attach_window_to_xaml(&window, &mut xaml_isle)
        .expect("Failed to create WinUI host control (HWND).");

    let size = window.inner_size();
    ui::update_xaml_island_size(&xaml_isle, size).expect("Couldn't update XAML Island HWND size.");

    unsafe {
        winapi::um::winuser::UpdateWindow(xaml_isle.hwnd_parent as winapi::shared::windef::HWND);
    }

    let browsers: Vec<os_browsers::Browser> =
        os_browsers::read_system_browsers_sync().expect("Could not read browser list");

    let list_items: Vec<ui::ListItem> = browsers
        .iter()
        .map(ui_list_item_from_browser)
        .rev()
        .collect();

    let ui_container = ui::create_ui(&ui::UI {
        browser_list: &list_items,
        event_loop: &event_loop_proxy,
        xaml_isle: &xaml_isle,
        url: &url,
    })
    .expect("Unable to create UI.");
    xaml_isle.desktop_source.set_content(ui_container).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                util::hide_window(&window);
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_size),
                ..
            } => {
                ui::update_xaml_island_size(&xaml_isle, _size);
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } if input.state == winit::event::ElementState::Pressed => {}
            Event::UserEvent(ui::BSEvent::Close) => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(ui::BSEvent::BrowserSelected(item_index)) => {
                let browser = &browsers[item_index as usize];
                os_browsers::open_url(&url, &browser);
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });

    Ok(())
}

fn ui_list_item_from_browser(browser: &os_browsers::Browser) -> ui::ListItem {
    use winapi::shared::windef::HICON;

    let app_hicon: HICON
        = os_browsers::get_exe_file_icon(browser.exe_path.as_str()).unwrap_or(0 as HICON);
    let software_bmp = ui::hicon_to_software_bitmap(app_hicon).unwrap();
    let xaml_image = ui::software_bitmap_to_xaml_image(software_bmp).unwrap();

    ui::ListItem {
        title: browser.version.product_name.clone(),
        subtitle: vec![
                browser.version.product_version.clone(),
                browser.version.binary_type.to_string(),
                browser.version.company_name.clone(),
                browser.version.file_description.clone(),
            ].into_iter()
            .filter(|itm| itm.len() > 0).collect::<Vec<String>>()
            .join(" | "),
        image: xaml_image
    }
}