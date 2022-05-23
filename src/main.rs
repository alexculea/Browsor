#[macro_use]
extern crate simple_error;

mod error;
mod os_util;
mod ui;

use ::std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::os_util::os_browsers;
use ui::{BrowserSelectorUI, UserInterface};

fn main() {
    std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicInfo| {
        crate::os_util::output_panic_text(panic_info.to_string());
        std::process::exit(1);
    }));

    let app_name = env!("CARGO_PKG_NAME");
    let app_version = env!("CARGO_PKG_VERSION");
    let arguments: Vec<String> = std::env::args().collect();
    let first_cli_argument = if arguments.len() >= 2 {
        arguments[1].to_owned() // arg[0] is executable path
    } else {
        String::new()
    };
    let cli_arg_open_url = first_cli_argument;

    // let env_name = std::env::var("ENV").unwrap_or("production".to_string());
    // let config_dir = os_util::get_create_config_directory("browser-selector", &env_name).unwrap_or(
    //     std::env::current_dir()
    //     .unwrap()
    //     .to_owned()
    //     .as_os_str()
    //     .to_string_lossy()
    //     .to_string()
    // );
    // TODO: Reintroduce when config is being used

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

    let browsers: Vec<os_browsers::Browser> =
        os_browsers::read_system_browsers_sync().expect("Could not read browser list");

    let list_items: Vec<ui::ListItem<os_browsers::Browser>> = browsers
        .iter()
        .map(ui_list_item_from_browser)
        .rev()
        .collect();

    ui.set_list(&list_items)
        .expect("Couldn't populate browsers in the UI.");
    ui.set_url(cli_arg_open_url.as_str())
        .expect("Couldn't render URL in the UI.");

    let open_url_clone = cli_arg_open_url.clone();
    ui.on_list_item_selected(move |uuid| {
        if let Some(item) = list_items.iter().find(|item| item.uuid == uuid) {
            // TODO: DRY refactor
            let mut command_arguments = item.state.arguments.clone();
            command_arguments.push(open_url_clone.clone());

            std::process::Command::new(&item.state.exe_path)
                .args(command_arguments)
                .spawn()
                .expect(
                    format!("Couldn't run browser program at {}", &item.state.exe_path)
                        .to_owned()
                        .as_str(),
                );

            std::process::exit(0);
        }
    })
    .expect("Cannot set on click event handler.");

    // to load the UI from a xaml file instead:
    // use winrt::ComInterface;
    // use bindings::windows::ui::xaml::markup::XamlReader;
    // use bindings::windows::ui::xaml::UIElement;
    // let xaml = fs::read_to_string("src\\main.xaml").expect("Cant read XAML file");
    // let ui_container = XamlReader::load(xaml).expect("Failed loading XAML").query::<UIElement>();
    window.set_visible(true);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            // TODO: This is the documented way to exit as per winit, however this crashes
            // Event::WindowEvent {
            //     event: WindowEvent::CloseRequested,
            //     window_id,
            // } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                std::process::exit(0);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_size),
                ..
            } => {
                ui.update_layout_size(&window, &_size).unwrap();
                // this causes a memory violation
                // when the program is closed but does work correclty
                // while the program is running
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } if input.state == winit::event::ElementState::Pressed => {
                use winit::event::VirtualKeyCode;
                let key = input
                    .virtual_keycode
                    .expect("Couldn't identify pressed key.");
                let length = ui
                    .get_list_length()
                    .expect("Couldn't determine list length")
                    - 1;
                let mut current_index = ui
                    .get_selected_list_item_index()
                    .expect("Couldn't determine currently selected item");

                match key {
                    VirtualKeyCode::Down => {
                        current_index = (current_index + 1).clamp(0, length as isize);
                        ui.select_list_item_by_index(current_index)
                            .expect("Couldn't select next item.");
                    }
                    VirtualKeyCode::Up => {
                        current_index = (current_index - 1).clamp(0, length as isize);
                        ui.select_list_item_by_index(current_index)
                            .expect("Couldn't select previous item.");
                    }
                    VirtualKeyCode::NumpadEnter | VirtualKeyCode::Return => {
                        let item = ui.get_selected_list_item().ok().unwrap().unwrap();

                        // TODO: DRY refactor
                        let mut command_arguments = item.state.arguments.clone();
                        command_arguments.push(cli_arg_open_url.clone());
                        std::process::Command::new(&item.state.exe_path)
                            .args(command_arguments)
                            .spawn()
                            .expect(
                                format!("Couldn't run browser program at {}", &item.state.exe_path)
                                    .to_owned()
                                    .as_str(),
                            );
                        std::process::exit(0);
                    }
                    VirtualKeyCode::Escape => {
                        std::process::exit(0);
                    }
                    vkey => {
                      if let Some(pos) = list_number_from_vkey(vkey) {
                        ui.select_list_item_by_index(pos.clamp(0, length as isize))
                            .expect("Couldn't select specific item number")
                      };
                    },
                }
            }
            _ => (),
        }
    });
}

fn ui_list_item_from_browser(browser: &os_browsers::Browser) -> ui::ListItem<os_browsers::Browser> {
    let image = BrowserSelectorUI::<os_browsers::Browser>::load_image(browser.exe_path.as_str())
        .unwrap_or_default();

    let uuid = {
        let mut hasher = DefaultHasher::new();
        browser.exe_path.hash(&mut hasher);
        hasher.finish().to_string()
    };

    ui::ListItem {
        title: browser.version.product_name.clone(),
        subtitle: vec![
            browser.version.product_version.clone(),
            browser.version.binary_type.to_string(),
            browser.version.company_name.clone(),
            browser.version.file_description.clone(),
        ]
        .into_iter()
        .filter(|itm| itm.len() > 0)
        .collect::<Vec<String>>()
        .join(" | "),
        image,
        uuid,
        state: std::rc::Rc::new(browser.clone()),
    }
}

fn list_number_from_vkey(vkey: winit::event::VirtualKeyCode) -> Option<isize> {
  use  winit::event::VirtualKeyCode;
  match vkey {
    VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => Some(0),
    VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => Some(1),
    VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => Some(2),
    VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => Some(3),
    VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => Some(4),
    VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => Some(5),
    VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => Some(6),
    VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => Some(7),
    VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => Some(8),
    _ => None
  }
}
