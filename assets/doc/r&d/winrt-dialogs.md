# Code snippet for WinRT dialog windows

Snippet is below but best to checkout the discussion in SO here https://stackoverflow.com/questions/62107050/how-can-i-create-a-messagedialog-using-winrt-rs

```Rust
trait InitializeWithWindow {
  fn initialize_with_window<O: RuntimeType + ComInterface>(
      &self,
      object: &O,
  ) -> winrt::Result<()>;
}

impl<T> InitializeWithWindow for T
where
  T: HasRawWindowHandle,
{
  fn initialize_with_window<O: RuntimeType + ComInterface>(
      &self,
      object: &O,
  ) -> winrt::Result<()> {
      // Get the window handle
      let window_handle = self.raw_window_handle();
      let window_handle = match window_handle {
          raw_window_handle::RawWindowHandle::Windows(window_handle) => window_handle.hwnd,
          _ => panic!("Unsupported platform!"),
      };

      let init: InitializeWithWindowInterop = object.try_into()?;
      init.initialize(window_handle)?;
      Ok(())
  }
}

eventHandler = move || -> {
  use bindings::windows::ui::popups::MessageDialog;
  let dialog = MessageDialog::create("Test").unwrap();
  window.initialize_with_window(&dialog).unwrap();
  dialog.show_async().unwrap();
  println!("KeyState-{}", input.scancode);
}
```