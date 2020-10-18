// WinRT imports
// the macro generates rust code calling the WinRT COM based on the winmd metadata supplied by the Windows SDK.

// Adding blanket imports with ::* can drastically increase compilation time
winrt::build!(
  dependencies
      os
  types
      windows::foundation::{PropertyValue}
      windows::storage::streams::{
        DataWriter, IDataWriterFactory, IBuffer
      }
      windows::ui::xaml::{UIElement, RoutedEventHandler, Thickness}
      windows::ui::xaml::controls::{
        Button, IButtonFactory, 
        IRelativePanelFactory, RelativePanel, 
        ListBox, IListBoxFactory, ScrollViewer, ScrollMode,
        TextBlock, 
        IListViewFactory, ListView, ListViewSelectionMode,
        IStackPanelFactory,
        StackPanel,
        Orientation,
        Image,
        Grid,
        ColumnDefinitions,
        RowDefinition,
        IGridFactory,
        GridUnitType,
        GridLength,
        IGridStatics
      }
      windows::ui::xaml::markup::*
      windows::ui::xaml::media::imaging::{
        SoftwareBitmapSource
      }
      windows::ui::xaml::hosting::{
        DesktopWindowXamlSource,
        IDesktopWindowXamlSourceFactory,
        WindowsXamlManager
      }
      windows::graphics::imaging::{
        SoftwareBitmap, ISoftwareBitmapFactory, BitmapPixelFormat, BitmapAlphaMode
      }
);

fn main() {
  build();
}
