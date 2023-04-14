use std::cell::RefCell;
use std::rc::Rc;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget};



use crate::os::sys_browsers::Browser;

pub enum UserEvent {
    Close,
}

pub fn make_ev_loop() -> EventLoop<UserEvent> {
    EventLoopBuilder::with_user_event().build()
}

pub fn make_runner<UIType>(
    url: Rc<String>,
    window: winit::window::Window,
    ui_ptr: Rc<RefCell<UIType>>,
    mut delegate: impl FnMut(&mut ControlFlow) -> (),
) -> impl FnMut(Event<UserEvent>, &EventLoopWindowTarget<UserEvent>, &mut ControlFlow) -> ()
where
    UIType: crate::ui::UserInterface<Browser>,
{
    move |event: Event<UserEvent>, _, control_flow: &mut ControlFlow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(10),
        );

        handle_ui_event(event, control_flow, &window, ui_ptr.clone(), url.clone());
        delegate(control_flow);
    }
}

pub fn handle_ui_event<UIType>(event: Event<UserEvent>, control_flow: &mut ControlFlow, window: &winit::window::Window, ui_ptr: Rc<RefCell<UIType>>, url: Rc<String>)
where
    UIType: crate::ui::UserInterface<Browser>,
{
    match event {
        Event::UserEvent(_) => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_size),
            ..
        } => {
            let ui = ui_ptr.borrow();
            ui.update_layout_size(window, &_size).unwrap();
        }
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } if input.state == winit::event::ElementState::Pressed => {
            use winit::event::VirtualKeyCode;
            let ui = ui_ptr.borrow();
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
                    let browser = item.state.as_ref();
                    crate::os::util::spawn_browser_process(
                        &browser.exe_path,
                        browser.arguments.clone(),
                        &url,
                    );

                    *control_flow = ControlFlow::Exit;
                }
                VirtualKeyCode::Space => {
                    if let Some(item) = ui.prediction_get_state().iter().take(1).last() {
                        let browser = item.state.as_ref();
                        crate::os::util::spawn_browser_process(
                            &browser.exe_path,
                            browser.arguments.clone(),
                            &url,
                        );
                        *control_flow = ControlFlow::Exit;
                    }
                }
                VirtualKeyCode::Back => {
                    if let Some(item) = ui.prediction_get_state().iter().take(2).last() {
                        let browser = item.state.as_ref();
                        crate::os::util::spawn_browser_process(
                            &browser.exe_path,
                            browser.arguments.clone(),
                            &url,
                        );
                        *control_flow = ControlFlow::Exit;
                    }
                }
                VirtualKeyCode::Escape => {
                    *control_flow = ControlFlow::Exit;
                }
                vkey => {
                    if let Some(pos) = list_number_from_vkey(vkey) {
                        ui.select_list_item_by_index(pos.clamp(0, length as isize))
                            .expect("Couldn't select specific item number");
                    };
                }
            }
        }
        _ => (),
    }

    if *control_flow == ControlFlow::Exit {
        window.set_visible(false);
        let ui = ui_ptr.borrow();
        ui.destroy();
    }
}

fn list_number_from_vkey(vkey: winit::event::VirtualKeyCode) -> Option<isize> {
    use winit::event::VirtualKeyCode;
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
        _ => None,
    }
}
