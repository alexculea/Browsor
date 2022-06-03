#[cfg(target_os = "windows")]
mod win;
pub mod ev_loop;

use crate::error::BSResult;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use std::rc::Rc;

/*
  This is a stub for seamlesly integrating multiple platforms (OSes)
*/
#[cfg(target_os = "windows")]
pub type Image = bindings::windows::ui::xaml::controls::Image;
#[cfg(target_os = "windows")]
pub use win::BrowserSelectorUI;
#[cfg(target_os = "windows")]
mod windows_desktop_window_xaml_source;

pub trait UserInterface<T: Clone> {
    fn new() -> BSResult<BrowserSelectorUI<T>>;
    fn create(&mut self, winit_wnd: &Window) -> BSResult<()>;

    fn set_list(&mut self, list: &[ListItem<T>]) -> BSResult<()>;
    fn set_url(&self, url: &str) -> BSResult<()>;

    fn update_layout_size(&self, window: &Window, size: &PhysicalSize<u32>) -> BSResult<()>;
    fn load_image(path: &str) -> BSResult<Image>;

    fn select_list_item_by_index(&self, index: isize) -> BSResult<()>;
    fn get_selected_list_item_index(&self) -> BSResult<isize>;
    fn get_selected_list_item(&self) -> BSResult<Option<ListItem<T>>>;
    fn get_list_length(&self) -> BSResult<usize>;

    fn on_list_item_selected(
        &self,
        event_handler: impl FnMut(&str) -> () + 'static,
    ) -> BSResult<()>;

    fn destroy(&self);
}

#[derive(Clone)]
pub struct ListItem<T: Clone> {
    pub title: String,
    pub subtitle: String,
    pub image: Image,
    pub uuid: String,
    pub state: Rc<T>,
}
