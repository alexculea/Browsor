#[cfg(target_os = "windows")]
mod win;
pub mod ev_loop;

use anyhow::Result;
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
    fn new() -> Result<BrowserSelectorUI<T>>;
    fn create(&mut self, winit_wnd: &Window) -> Result<()>;

    fn set_list(&mut self, list: &[ListItem<T>]) -> Result<()>;
    fn set_url(&self, url: &str) -> Result<()>;

    fn update_layout_size(&self, window: &Window, size: &PhysicalSize<u32>) -> Result<()>;
    fn load_image(path: &str) -> Result<Image>;

    fn select_list_item_by_index(&self, index: isize) -> Result<()>;
    fn get_selected_list_item_index(&self) -> Result<isize>;
    fn get_selected_list_item(&self) -> Result<Option<ListItem<T>>>;
    fn get_list_length(&self) -> Result<usize>;

    fn on_list_item_selected(
        &self,
        event_handler: impl FnMut(&str) -> () + 'static,
    ) -> Result<()>;

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
