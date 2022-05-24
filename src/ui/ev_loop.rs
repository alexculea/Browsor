use winit::event_loop::ControlFlow;
use winit::event::WindowEvent;
use winit::event::Event;
use winit::event_loop::EventLoopWindowTarget;

use crate::os::sys_browsers::Browser;

pub fn make_ev_loop<UIType>(
  url: String,
  window: winit::window::Window,
  ui: UIType,
) -> impl FnMut(Event<()>, &EventLoopWindowTarget<()>, &mut ControlFlow) -> ()
where
  UIType: crate::ui::UserInterface<Browser>
{
  move |event: Event<()>, _, control_flow: &mut ControlFlow| {
    *control_flow = ControlFlow::Wait;
    match event {
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
                    let browser = item.state.as_ref();
                    crate::os::util::spawn_and_exit(&browser.exe_path, browser.arguments.clone(), &url);
                }
                VirtualKeyCode::Escape => {
                    std::process::exit(0);
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