#![windows_subsystem = "windows"]
#[macro_use]
extern crate anyhow;

mod conf;
mod error;
mod os;
mod ui;

use std::rc::Rc;
use winit::window::WindowBuilder;

use crate::os::sys_browsers;
use crate::os::sys_browsers::Browser;
use crate::ui::{BrowserSelectorUI, ListItem, UserInterface};

fn main() {
    std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicInfo| {
        crate::os::output_panic_text(panic_info.to_string());
        std::process::exit(1);
    }));

    let config = conf::read_config().unwrap_or_default();
    let app_name = env!("CARGO_PKG_NAME");
    let app_version = env!("CARGO_PKG_VERSION");
    let target_url = Rc::new(std::env::args().nth(1).unwrap_or(config.default_url));

    let mut ui = BrowserSelectorUI::new().expect("Failed to initialize COM or WinUI");
    let event_loop = ui::ev_loop::make_ev_loop();
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
        .filter(|browser| {
            let hide_browser = config.hide.iter().fold(false, |_, conf_hide_item| {
                let hide_by_path = !conf_hide_item.path.is_empty()
                    && browser.exe_path.contains(&conf_hide_item.path);
                let hide_by_name =
                    !conf_hide_item.name.is_empty() && browser.name.contains(&conf_hide_item.name);

                hide_by_name || hide_by_path
            });

            !hide_browser // invert for filter as true = keep item, false = discard item from list
        })
        .filter_map(|item| item.try_into().ok())
        .collect();

    ui.set_list(&list_items)
        .expect("Couldn't populate browsers in the UI.");
    ui.set_url(&target_url)
        .expect("Couldn't render URL in the UI.");

    let open_url_clone = Rc::clone(&target_url);
    let ev_loop_proxy = event_loop.create_proxy();
    ui.on_list_item_selected(move |uuid| {
        list_items
            .iter()
            .find(|item| item.uuid == uuid)
            .and_then(|item| Some(item.state.as_ref()))
            .and_then::<std::rc::Rc<Browser>, _>(|browser| {
                os::util::spawn_browser_process(
                    &browser.exe_path,
                    browser.arguments.clone(),
                    &open_url_clone,
                );
                None
            });

        ev_loop_proxy.send_event(ui::ev_loop::UserEvent::Close).ok();
    })
    .expect("Cannot set on click event handler.");

    window.set_visible(true);
    event_loop.run(ui::ev_loop::make_runner(target_url, window, ui));
}
