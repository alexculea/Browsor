// WinRT imports
// the macro generates rust code calling the WinRT COM based on the winmd metadata supplied by the Windows SDK.
winrt::build!(
  dependencies
      os
  types
      windows::foundation::{PropertyValue}
      windows::ui::xaml::*
      windows::ui::xaml::controls::{
        Button, IButtonFactory, IRelativePanelFactory, RelativePanel,
      }
      windows::ui::xaml::hosting::{
        DesktopWindowXamlSource, 
        IDesktopWindowXamlSourceFactory, 
        WindowsXamlManager
      }
      windows::ui::popups::*
);

fn main() {
  build();
}