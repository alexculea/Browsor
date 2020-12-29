mod ui_windows;

use winit::window::Window;
use winit::dpi::PhysicalSize;
use crate::error::BSResult;

/*
  This is a stub for seamlesly integrating multiple platforms (OSes)

*/
#[cfg(target_os = "windows")]
pub type Image = bindings::windows::ui::xaml::controls::Image;
#[cfg(target_os = "windows")]
pub use ui_windows::BrowserSelectorUI;


pub trait UserInterface {
  fn new() -> BSResult<BrowserSelectorUI>;
  fn create(&mut self, winit_wnd: &Window) -> BSResult<()>;
  
  fn set_list(&self, list: &Vec<ListItem>) -> BSResult<()>;
  fn set_url(&self, url: &str) -> BSResult<()>;

  fn update_layout_size(&self, window: &Window, size: &PhysicalSize<u32>) -> BSResult<()>;
  fn load_image(path: &str) -> BSResult<Image>;

  fn select_list_item_by_index(&self, index: u32) -> BSResult<()>;
  fn select_list_item_by_uuid(&self, uuid: &str) -> BSResult<()>;
  fn get_selected_list_item_index(&self) -> BSResult<u32>;
  fn get_selected_list_item(&self) -> BSResult<Option<ListItem>>;

  fn on_list_item_selected(&self, event_handler: fn(&ListItem) -> ()) -> BSResult<()>;
}

#[derive(Clone)]
pub struct ListItem {
    pub title: String,
    pub subtitle: String,
    pub image: Image,
    pub uuid: u64
}
