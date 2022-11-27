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

type BuildResult = Result<(), std::io::Error>;
fn main() -> BuildResult {
    windows()
}

fn windows() -> BuildResult {
    // OS Metdata directory: let metadata_dir = format!("{}\\System32\\WinMetadata", env!("windir"));
    // Q: Is it better to use the OS metadata or bundle a fixed version?
    println!("cargo:rerun-if-changed=build.rs");
    use windows_metadata::reader::{File, Reader};
    // use windows_bindgen::{component};
    const WINMD_FILES: &[&str] = &[
        "Windows.winmd",
    ];

    let imports = vec!(
        "Windows.UI.Xaml",
        "Windows.UI.Xaml.Controls",
        // "Windows.UI.Xaml.Maps",
        // "Windows.UI.Xaml.Primitives",
        // "Windows.UI.Xaml.Colors",
        "Windows.UI.Xaml.Media",
        "Windows.UI.Xaml.Media.Imaging",
        "Windows.UI.Xaml.Hosting",
    );
    
    // let imports = vec!(
    //     "Windows.UI.Xaml",
    //     "Windows.UI.Xaml.Hosting",
    //     "Windows.UI.Xaml.Media",
    //     "Windows.UI.Xaml.Media.Imaging",
    // );

    // let windows_rs_modules = [
    //     ExternalModule::new("Accessibility", "::windows::UI::Accessibility"),
    //     ExternalModule::new("ApplicationSettings", "::windows::UI::ApplicationSettings"),
    //     ExternalModule::new("Composition", "::windows::UI::Composition"),
    //     ExternalModule::new("WindowManagement", "::windows::UI::Core"),
    //     ExternalModule::new("WindowManagement", "::windows::UI::Input"),
    // ];

    let cargo_out_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
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
    for namespace in imports {
        gen_tree(&metadata_reader, &cargo_out_dir, &metadata_reader.tree(namespace, &[]).unwrap(), None)
    }

    rewrite_missing_modules_to_imports(&cargo_out_dir, "crate::windows::")?;

    // let components = component("Windows.Foundation", &winmd_files);
    // println!("{:?}", components);

    // let tree = metadata_reader.tree("Windows.UI", &[]).unwrap();

    // let mut gen = Gen::new(&metadata_reader);
    // gen.namespace = tree.namespace;

    // let mut source_path = cargo_out_dir;
    // source_path.push("mod.rs");
    // let mut source_file = std::fs::File::create(source_path.clone()).unwrap();

    // source_file.write_all(namespace(&gen, &tree).as_bytes()).unwrap();
    // source_file.write_all(namespace_impl(&gen, &tree).as_bytes()).unwrap();

    Ok(())
}

// fn windows_cmpt() -> i32 {
//     // OS Metdata directory: let metadata_dir = format!("{}\\System32\\WinMetadata", env!("windir"));
//     // Q: Is it better to use the OS metadata or bundle a fixed version?
//     println!("cargo:rerun-if-changed=build.rs");
//     use crate::bindings::windows_metadata::reader::{File, Reader};
//     use windows_bindgen::{component};
//     const WINMD_FILES: &[&str] = &[
//         "Windows.winmd",
//     ];

//     let imports = vec!(
//         // "Windows.Foundation.Collections",
//         // "Windows.Foundation.Diagnostics",
//         // "Windows.Foundation.Numerics",
//         // "Windows.Foundation.Metadata",
//         // "Windows.Storage",
//         "Windows.Foundation"
//     );

//     let cargo_out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
//     let mut winmd_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
//     winmd_path.push("winmd");
//     let winmd_files: Vec<_> = WINMD_FILES
//         .iter()
//         .map(|name| {
//             let mut winmd_path = winmd_path.clone();
//             winmd_path.push(name);
//             let winmd_path = winmd_path.to_str().expect("invalid winmd path");
//             File::new(winmd_path).expect(name)
//         })
//         .collect();
//     //let metadata_reader = Reader::new(&winmd_files);
//     for namespace in imports {
//         std::fs::write(cargo_out_dir.clone().join("mod.rs"), component(namespace, &winmd_files)).unwrap();
//     }

//     // let components = component("Windows.Foundation", &winmd_files);
//     // println!("{:?}", components);

//     // let tree = metadata_reader.tree("Windows.UI", &[]).unwrap();

//     // let mut gen = Gen::new(&metadata_reader);
//     // gen.namespace = tree.namespace;

//     // let mut source_path = cargo_out_dir;
//     // source_path.push("mod.rs");
//     // let mut source_file = std::fs::File::create(source_path.clone()).unwrap();

//     // source_file.write_all(namespace(&gen, &tree).as_bytes()).unwrap();
//     // source_file.write_all(namespace_impl(&gen, &tree).as_bytes()).unwrap();

//     return 0;
// }

struct ExternalModule<'a> {
    from: &'a str,
    to: &'a str,
}

impl<'a> ExternalModule<'a> {
    pub fn new(from: &'a str, to: &'a str) -> ExternalModule<'a> {
        ExternalModule { from, to }
    }
}

fn gen_tree(reader: &windows_metadata::reader::Reader, output: &std::path::Path, tree: &windows_metadata::reader::Tree, externals: Option<&[ExternalModule]>) {
    use windows_bindgen::{Gen, namespace, namespace_impl};

    println!("{}", tree.namespace);
    let mut path = std::path::PathBuf::from(output);
    path.push(tree.namespace.replace('.', "/"));
    std::fs::create_dir_all(&path).unwrap();

    let mut gen = Gen::new(reader);
    gen.namespace = tree.namespace;
    gen.cfg = true;
    gen.doc = true;
    gen.component = true;
    let mut tokens = namespace(&gen, tree);
    tokens.push_str(r#"#[cfg(feature = "implement")] ::core::include!("impl.rs");"#);
    std::fs::write(path.join("mod.rs"), tokens).unwrap();
    let tokens = namespace_impl(&gen, tree);
    
    std::fs::write(path.join("impl.rs"), tokens).unwrap();
}

fn rewrite_missing_modules_to_imports<'a>(root_module_path: &std::path::PathBuf, import_prefix: &str) -> BuildResult {
    // root module path = ./out/Windows/UI/Xaml/mod.rs
    // read modules form the root module
    // for each module
    //      if the module exists
                // rewrite the its missing modules recursively
    //      if the module does not exist
    //          rewrite module with import prefix



    //  use regex::Regex;
    //  let re = Regex::new(r"pub mod ([A-z0-9_]+);").unwrap();
     
    //  for dir_entry in std::fs::read_dir(root_path) {
    //     if (dir_entry.is_dir())

    //     for re_match in re.captures_iter(&"pub mod Hello;") {
    //         let module_name = &re_match[1];
    //         println!("{:?}", module_name);
        
    //      }
    //  }

     

     Ok(())
}

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}