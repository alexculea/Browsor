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

fn windows() -> i32 {
    // OS Metdata directory: let metadata_dir = format!("{}\\System32\\WinMetadata", env!("windir"));
    // Q: Is it better to use the OS metadata or bundle a fixed version?
    println!("cargo:rerun-if-changed=build.rs");
    use windows_metadata::reader::{File, Reader};
    use windows_bindgen::{Gen, namespace, namespace_impl};
    use std::io::Write;
    const WINMD_FILES: &[&str] = &[
        "Windows.winmd",
    ];

    // let imports = vec!(
    //     "Windows.UI.Xaml.UIElement",
    //     "Windows.UI.Xaml.RoutedEventHandler",
    //     "Windows.UI.Xaml.Thickness",
    //     "Windows.UI.Xaml.Controls",
    //     "Windows.UI.Xaml.Colors",
    //     "Windows.UI.ViewManagement.UISettings",
    //     "Windows.UI.Xaml.Media.Imaging.SoftwareBitmapSource",
    //     "Windows.UI.Xaml.Hosting",
    // );
    
    let imports = vec!(
        "Windows.UI",
        "Windows.UI.Xaml",
    );

    let cargo_out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let mut winmd_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
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
    for ns in imports {
        gen_tree(&metadata_reader, &cargo_out_dir, &metadata_reader.tree(ns, &[]).unwrap())
    }

    // let tree = metadata_reader.tree("Windows.UI", &[]).unwrap();

    // let mut gen = Gen::new(&metadata_reader);
    // gen.namespace = tree.namespace;

    // let mut source_path = cargo_out_dir;
    // source_path.push("mod.rs");
    // let mut source_file = std::fs::File::create(source_path.clone()).unwrap();

    // source_file.write_all(namespace(&gen, &tree).as_bytes()).unwrap();
    // source_file.write_all(namespace_impl(&gen, &tree).as_bytes()).unwrap();

    return 0;
}

fn gen_tree(reader: &windows_metadata::reader::Reader, output: &std::path::Path, tree: &windows_metadata::reader::Tree) {
    use windows_bindgen::{Gen, namespace, namespace_impl};

    println!("{}", tree.namespace);
    let mut path = std::path::PathBuf::from(output);
    path.push(tree.namespace.replace('.', "/"));
    std::fs::create_dir_all(&path).unwrap();

    let mut gen = Gen::new(reader);
    gen.namespace = tree.namespace;
    gen.cfg = true;
    gen.doc = true;
    let mut tokens = namespace(&gen, tree);
    tokens.push_str(r#"#[cfg(feature = "implement")] ::core::include!("impl.rs");"#);
    
    std::fs::write(path.join("mod.rs"), tokens).unwrap();
    let tokens = namespace_impl(&gen, tree);
    
    std::fs::write(path.join("impl.rs"), tokens).unwrap();
}
