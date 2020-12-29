#[macro_use]
extern crate simple_error;

mod desktop_window_xaml_source;
mod os_browsers;
mod ui;
mod os_util;
mod error;

use::std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use ui::{BrowserSelectorUI, UserInterface};

fn main() {
    let mut ui = BrowserSelectorUI::new().expect("Failed to initialize COM or WinUI");
    let url: String = "http://www.google.com".into();

    let event_loop = EventLoop::new();
    // let event_loop_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    ui.create(&window).expect("Failed to initialize WinUI XAML.");

    let browsers: Vec<os_browsers::Browser> =
        os_browsers::read_system_browsers_sync().expect("Could not read browser list");

    let list_items: Vec<ui::ListItem> = browsers
        .iter()
        .map(ui_list_item_from_browser)
        .rev()
        .collect();

    ui.set_list(&list_items).expect("Couldn't populate browsers in the UI.");
    ui.set_url(url.as_str()).expect("Couldn't render URL in the UI.");

    // to load the UI from a xaml file instead:
    // use winrt::ComInterface;
    // use bindings::windows::ui::xaml::markup::XamlReader;
    // use bindings::windows::ui::xaml::UIElement;
    // let xaml = fs::read_to_string("src\\main.xaml").expect("Cant read XAML file");
    // let ui_container = XamlReader::load(xaml).expect("Failed loading XAML").query::<UIElement>();



    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_size),
                ..
            } => {
                ui.update_layout_size(&window, &_size);
                // this causes a memory violation
                // when the program is closed but does work correclty
                // while the program is running
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } if input.state == winit::event::ElementState::Pressed => {}
            _ => (),
        }
    });

    Ok(())
}

fn ui_list_item_from_browser(browser: &os_browsers::Browser) -> ui::ListItem {
    let image = BrowserSelectorUI::load_image(browser.exe_path.as_str())
        .unwrap_or_default();
    
    let uuid = {
        let mut hasher = DefaultHasher::new();
        browser.exe_path.hash(&mut hasher);
        hasher.finish()
    };

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
        image,
        uuid,
    }
}