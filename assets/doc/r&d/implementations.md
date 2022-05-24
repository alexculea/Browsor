### Resources for future implementations
https://stackoverflow.com/questions/62107050/how-can-i-create-a-messagedialog-using-winrt-rs
https://blogs.windows.com/windowsdeveloper/2019/04/30/enhancing-non-packaged-desktop-apps-using-windows-runtime-components/


#### XML Files UI 
Makes a control from .xml file.
```C++
winrt::Windows::UI::Xaml::UIElement LoadXamlControl(uint32_t id)
{
    auto rc = ::FindResource(nullptr, MAKEINTRESOURCE(id), MAKEINTRESOURCE(XAMLRESOURCE));
    if (!rc)
    {
        winrt::check_hresult(HRESULT_FROM_WIN32(GetLastError()));
    }
    HGLOBAL rcData = ::LoadResource(nullptr, rc);
    if (!rcData)
    {
        winrt::check_hresult(HRESULT_FROM_WIN32(GetLastError()));
    }
    auto pData = static_cast<wchar_t*>(::LockResource(rcData));
    auto content = winrt::Windows::UI::Xaml::Markup::XamlReader::Load(winrt::get_abi(pData));
    auto uiElement = content.as<winrt::Windows::UI::Xaml::UIElement>();
    return uiElement;
}
```
taken from <https://github.com/microsoft/Xaml-Islands-Samples/tree/master/Samples/Win32/SampleCppApp>

Rust variant:
```Rust
    use winrt::ComInterface;
    use bindings::windows::ui::xaml::markup::XamlReader;
    use bindings::windows::ui::xaml::UIElement;
    
    [...]

    let xaml = fs::read_to_string("src\\main.xaml").expect("Cant read XAML file");
    let ui_container = XamlReader::load(xaml).expect("Failed loading XAML").query::<UIElement>();
```

Loading resources at run time:
[https://stackoverflow.com/questions/2933295/embed-text-file-in-a-resource-in-a-native-windows-application]

#### Loading an WinRT Image from a incon 

[https://stackoverflow.com/questions/32122679/getting-icon-of-modern-windows-app-from-a-desktop-application]


### Get Icon from file?
https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shgetfileinfoa


### Receive strings from winapi

```Rust
let mut v: Vec<u16> = Vec::with_capacity(255);
unsafe {
    let read_len = user32::GetWindowTextW(
        handle as winapi::HWND,
        v.as_mut_ptr(),
        v.capacity(),
    );
    v.set_len(read_len); // this is undefined behavior if read_len > v.capacity()
    String::from_utf16_lossy(&v)
}
```