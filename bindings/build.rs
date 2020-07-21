// WinRT imports
// the macro generates rust code calling the WinRT COM based on the winmd metadata supplied by the Windows SDK.

// Adding blanket imports with ::* can drastically increase compilation time
// best to take only what we need
winrt::build!(
  dependencies
      os
  types
      windows::foundation::{PropertyValue}
      windows::ui::xaml::{UIElement, RoutedEventHandler, Thickness}
      windows::ui::xaml::controls::{
        Button, IButtonFactory, IRelativePanelFactory, RelativePanel, ListBox, IListBoxFactory, TextBlock,
      }
      windows::ui::xaml::hosting::{
        DesktopWindowXamlSource, 
        IDesktopWindowXamlSourceFactory, 
        WindowsXamlManager
      }
      // windows::ui::popups::*
);

fn main() {
  build();
}