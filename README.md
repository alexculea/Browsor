# Browser Selector
Choose the browser you will run any time a link is opened from a click or any other action.

# Requirements for first release
- As a user I:
  - start the program and be guided to set it as default OS browser
  - click a link and instead of a browser opening I see the program showing a list of browsers 
  - select browsers from the list by using my keyboard arrows or the mouse scroll wheel
  - don't have to manually focus the window
  - will always see the window on top of anything else
  - can get rid of the window if I either: hit ESC, click somewhere else
  - only the use program on Windows
  - benefit from a performant operation taking at most 300msec for the window to come up on screen when I use an SSD as my system drive
  - see list ordered by how often I use a certain browser

  UI Mockup     
  ![]( assets/ui-mock-version-1.0.svg )


# Future roadmap
- Allow manual drag and drop reordering of the list
- Support multiple Firefox profiles

# Far future roadmap
- Add support for Linux and MacOS
- Add learning algorithm that predicts the choice made by looking at: time of the day, location of the device, program from where the link was clicked


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


#### Loading an WinRT Image from a incon 

<https://stackoverflow.com/questions/32122679/getting-icon-of-modern-windows-app-from-a-desktop-application>


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