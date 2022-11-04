use bindings::windows::ui::xaml::hosting::DesktopWindowXamlSource;
use core::ffi::c_void;
use core::ptr;

#[repr(transparent)]
#[derive(PartialEq, Clone)]
pub struct IDesktopWindowXamlSourceNative(::windows::core::IUnknown);
impl IDesktopWindowXamlSourceNative {
    pub fn attach_to_window(&self, hwnd: *mut core::ffi::c_void) -> windows::core::Result<()> {
        let this = windows::core::Vtable::vtable(self);
        let this_ptr = ::windows::core::Vtable::as_raw(self);

        return (this.attach_to_window)(this_ptr, hwnd).ok();
    }

    pub fn get_window_handle(&self) -> windows::core::Result<*mut c_void> {
        let this = windows::core::Vtable::vtable(self);
        let this_ptr = ::windows::core::Vtable::as_raw(self);

        let mut hwnd = ptr::null_mut();
        return (this.get_window_handle)(this_ptr, &mut hwnd).and_then(|| hwnd);
    }
}

unsafe impl ::windows::core::Interface for IDesktopWindowXamlSourceNative {
    const IID: windows::core::GUID = ::windows::core::GUID::from_values(
        0x3cbcf1bf,
        0x2f76,
        0x4e9c,
        [0x96, 0xab, 0xe8, 0x4b, 0x37, 0x97, 0x25, 0x54],
    );
}

unsafe impl ::windows::core::Vtable for IDesktopWindowXamlSourceNative {
    type Vtable = IDesktopWindowXamlSourceNative_Vtbl;
}
// unsafe impl ::core::marker::Send for IDesktopWindowXamlSourceNative {}
// unsafe impl ::core::marker::Sync for IDesktopWindowXamlSourceNative {}

// unsafe impl ::winrt::AbiTransferable for IDesktopWindowXamlSourceNative {
//     type Abi = winrt::RawComPtr<Self>;

//     fn get_abi(&self) -> Self::Abi {
//         self.ptr.get_abi()
//     }

//     fn set_abi(&mut self) -> *mut Self::Abi {
//         self.ptr.set_abi()
//     }
// }

#[repr(C)]
pub struct IDesktopWindowXamlSourceNative_Vtbl {
    // pub unknown_query_interface: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>, &::winrt::Guid, *mut ::winrt::RawPtr) -> ::winrt::ErrorCode,
    // pub unknown_add_ref: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>) -> u32,
    // pub unknown_release: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>) -> u32,
    __base: [usize; 3], // leave 3 ptr spaces empty for the IUnknown
    pub attach_to_window: extern "system" fn(
        *mut ::core::ffi::c_void,
        *mut c_void,
    ) -> windows::core::HRESULT,
    pub get_window_handle: extern "system" fn(
        *mut ::core::ffi::c_void, // this
        *mut *mut c_void, // HWND
    ) -> windows::core::HRESULT,
}

impl From<&DesktopWindowXamlSource> for IDesktopWindowXamlSourceNative {
    fn from(value: &DesktopWindowXamlSource) -> IDesktopWindowXamlSourceNative {
        value.into()
    }
}

impl From<DesktopWindowXamlSource> for IDesktopWindowXamlSourceNative {
    fn from(value: DesktopWindowXamlSource) -> IDesktopWindowXamlSourceNative {
        std::convert::From::from(&value)
    }
}
