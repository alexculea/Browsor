// // WinRT imports
// // the macro generates rust code calling the WinRT COM based on the winmd metadata supplied by the Windows SDK.

// // Adding blanket imports with ::* can drastically increase compilation time
// winrt::build!(
//   dependencies
//       os
//   types
//       windows::foundation::{PropertyValue}
//       windows::storage::streams::{
//         DataWriter, IDataWriterFactory, IBuffer
//       }
//       windows::ui::xaml::{UIElement, RoutedEventHandler, Thickness}
//       windows::ui::xaml::controls::{
//         Button, IButtonFactory,
//         IRelativePanelFactory, RelativePanel,
//         ListBox, IListBoxFactory, ScrollViewer, ScrollMode,
//         TextBlock,
//         IListViewFactory, ListView, ListViewSelectionMode,
//         IStackPanelFactory,
//         StackPanel,
//         Orientation,
//         Image,
//         Grid,
//         ColumnDefinitions,
//         RowDefinition,
//         IGridFactory,
//         GridUnitType,
//         GridLength,
//         IGridStatics
//       }
//       windows::ui::xaml::media::imaging::{
//         SoftwareBitmapSource
//       }
//       windows::ui::xaml::hosting::{
//         DesktopWindowXamlSource,
//         IDesktopWindowXamlSourceFactory,
//         WindowsXamlManager
//       }
//       windows::graphics::imaging::{
//         SoftwareBitmap, ISoftwareBitmapFactory, BitmapPixelFormat, BitmapAlphaMode
//       }
//       windows::ui::Colors
//       windows::ui::view_management::{UISettings}
// );

fn main() {
    windows();
}

fn windows() {
    const WINMD_FILES: &[&str] = &[
        "Windows.winmd",
        "Windows.Win32.winmd",
        "Windows.Win32.Interop.winmd",
        "Microsoft.Web.WebView2.Win32.winmd",
    ];

    let carg_out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut winmd_path = get_manifest_dir()?;
    winmd_path.push("winmd");
    let winmd_files: Vec<_> = WINMD_FILES
        .iter()
        .map(|name| {
            let mut winmd_path = winmd_path.clone();
            winmd_path.push(name);
            let winmd_path = winmd_path.to_str().expect("invalid winmd path");
            File::new(winmd_path).expect(name)
        })
        .collect();
    let metadata_reader = Reader::new(&winmd_files);
    let tree = metadata_reader
        .tree("Microsoft.Web.WebView2.Win32", &[])
        .map_or_else(|| Err(super::Error::MissingPath(winmd_path)), Ok)?;
    let mut gen = Gen::new(&metadata_reader);
    gen.namespace = tree.namespace;

    let mut source_path = get_out_dir()?;
    source_path.push("mod.rs");
    let mut source_file = fs::File::create(source_path.clone())?;

    source_file.write_all(patch_bindings(namespace(&gen, &tree))?.as_bytes())?;
    source_file.write_all(namespace_impl(&gen, &tree).as_bytes())?;
    Ok(source_path)
}
