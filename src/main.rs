#[macro_use]
extern crate simple_error;

mod desktop_window_xaml_source;
mod os_browsers;
mod ui;
mod util;

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
    let mut xaml_isle = ui::XamlIslandWindow::default();
    ui::init_win_ui_xaml(&mut xaml_isle)
        .expect("Failed to initialize WinUI XAML.");

    let event_loop = EventLoop::with_user_event();
    let event_loop_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    xaml_isle.hwnd = ui::attach_window_to_xaml(&window, &mut xaml_isle)
        .expect("Failed to create WinUI host control (HWND).");

    let size = window.inner_size();
    ui::update_xaml_island_size(&xaml_isle, size);

    unsafe {
        winapi::um::winuser::UpdateWindow(xaml_isle.hwnd_parent as winapi::shared::windef::HWND);
    }

    let browsers: Vec<os_browsers::Browser> =
        os_browsers::read_system_browsers_sync().expect("Could not read browser list");

    let list_items: Vec<ui::ListItem> = browsers
        .iter()
        .map(move | browser_entry | { ui::ListItem { title: &browser_entry.name, subtitle: "Version placeholder" } } )
        .rev()
        .collect();

    let ui_container = ui::create_ui(&ui::UI { 
        browser_list: &list_items,
        event_loop: &event_loop_proxy,
        xaml_isle: &xaml_isle,
        url: &url,
    }).expect("Unable to create UI.");
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
            } if input.state == winit::event::ElementState::Pressed => {},
            Event::UserEvent(ui::BSEvent::Close) => {
                *control_flow = ControlFlow::Exit;
            },
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
