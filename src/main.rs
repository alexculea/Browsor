#![windows_subsystem = "windows"]
#[macro_use]
extern crate simple_error;

#[macro_use]
extern crate rusqlite;

mod conf;
mod data;
mod error;
mod os;
mod ui;

use core::cell::RefCell;
use os::ActiveWindowInfo;
use std::rc::Rc;
use winit::window::WindowBuilder;

use crate::os::sys_browsers;
use crate::os::sys_browsers::Browser;
use crate::ui::{BrowserSelectorUI, ListItem, UserInterface};

fn main() {
    std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicInfo| {
        crate::os::output_panic_text(panic_info.to_string());
    }));

    let config = conf::read_config().unwrap_or_default();
    let app_name = env!("CARGO_PKG_NAME");
    let app_version = env!("CARGO_PKG_VERSION");
    let target_url = Rc::new(
        std::env::args()
            .nth(1)
            .unwrap_or(config.default_url),
    );
    let mut statistics_optional: Option<Rc<RefCell<data::Statistics>>> = None;

    let ui_ptr = Rc::new(RefCell::new(
        BrowserSelectorUI::new().expect("Failed to initialize COM or WinUI"),
    ));
    let event_loop = ui::ev_loop::make_ev_loop();
    let mut browsers: Box<Vec<Browser>> = Box::new(Default::default());
    let mut src_app_opt: Option<ActiveWindowInfo> = None;

    if config.statistics {
        src_app_opt = Some(os::get_active_window_info());
        statistics_optional = Some(Rc::new(RefCell::new(data::Statistics::new())));
        let statistics_ref = statistics_optional.clone().unwrap();
        let mut statistics = statistics_ref.borrow_mut();
        let mut statistics_db_path = std::env::current_exe().unwrap_or_default();
        statistics_db_path.set_file_name("statistics.sqlite");
        statistics.set_db_path(&statistics_db_path);

        statistics.migrate_async(|res| {
            if res.is_ok() {
                println!("Migration finished")
            } else {
                let msg = format!("Migration failed, {}", res.err().unwrap());
                crate::os::output_panic_text(msg.to_string());
            }
        });
    }

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

    {
        let mut ui = ui_ptr.borrow_mut();
        ui.create(&window)
            .expect("Failed to initialize WinUI XAML.");
    }
    
    *browsers = sys_browsers::read_system_browsers_sync().expect("Could not read browser list");

    let selections = browsers
        .iter()
        .map(|browser| -> data::SelectionEntity {
            data::SelectionEntity {
                id: None,
                path: Some(browser.exe_path.clone()),
                path_hash: Some(browser.get_hash()),
            }
        })
        .collect();

    if let Some(stats) = statistics_optional.clone() {
        { let ui = ui_ptr.borrow(); ui.prediction_set_is_loading(true).unwrap(); }
        let mut statistics = stats.borrow_mut();
        statistics.update_selections(selections, |res| {
            if res.is_err() {
                let msg = format!(
                    "Failed updating the browsers available on the system to the statistics database.\nError:{}", res.err().unwrap()
                );
                crate::os::output_panic_text(msg.to_string());
            }
        });

        let source = src_app_opt.clone().unwrap_or_default().exe_path;
        let start_time = std::time::Instant::now();
        let browsers = browsers.clone();
        let ui_ptr = Rc::clone(&ui_ptr);
        statistics.predict(source, &target_url, move |result| {
            if let Ok(predicted_list) = result.as_ref() {
                let list = predicted_list
                    .iter()
                    .take(2)
                    .filter_map(|item: &data::SelectionEntity| {
                        let browser_opt = browsers.iter().find(|browser| {
                            browser.get_hash().as_str() == item.path_hash.as_ref().unwrap().as_str()
                        });
                        if let Some(browser) = browser_opt {
                            Some(browser.try_into().unwrap())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<ListItem<Browser>>>();
                let duration = start_time.elapsed();
                let duration_msec = duration.as_millis();
                let duration_str = format!("{} ms", duration_msec);
                ui_ptr
                    .borrow_mut()
                    .prediction_set_state(&list.as_slice(), &duration_str)
                    .unwrap();
            } else {
                ui_ptr.borrow().prediction_set_is_loading(false).unwrap();
            }
        });
    }

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

    {
        let mut ui = ui_ptr.borrow_mut();
        ui.set_list(&list_items)
            .expect("Couldn't populate browsers in the UI.");
        ui.set_url(&target_url)
            .expect("Couldn't render URL in the UI.");
    }

    let open_url_clone = Rc::clone(&target_url);
    let ev_loop_proxy = event_loop.create_proxy();

    let statistics_ref = statistics_optional.clone();
    let src_app_clone = src_app_opt.clone();

    { 
        let ui = ui_ptr.borrow(); 
        ui.on_list_item_selected(move |uuid| {
            let source = src_app_clone.clone().unwrap_or_default().exe_path;
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

                    if let Some(stats) = statistics_ref.clone() {
                        let browser_hash = browser.get_hash();
                        stats.borrow_mut().save_choice(
                            source,
                            &open_url_clone,
                            &browser_hash,
                            &browser.exe_path,
                            |res| {
                                if res.is_err() {
                                    let msg =
                                        format!("Failed saving choice.\nError:{}", res.err().unwrap());
                                    crate::os::output_panic_text(msg.to_string());
                                }
                            },
                        );
                    }

                    None
                });
            ev_loop_proxy.send_event(ui::ev_loop::UserEvent::Close).ok();
        })
        .expect("Cannot set on click event handler.");
    }

    window.set_visible(true);
    // drop(ui); // doesn't actually destroy the UI, just releases the borrow_mut()
    event_loop.run(ui::ev_loop::make_runner(
        target_url,
        window,
        ui_ptr.clone(),
        statistics_optional,
    ));
}
