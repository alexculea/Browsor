use core::ffi::c_void;
use core::ptr;
use bindings::windows::ui::xaml::hosting::DesktopWindowXamlSource;
use winrt::AbiTransferable;

#[repr(transparent)]
#[derive(Default, PartialEq, Clone)]
pub struct IDesktopWindowXamlSourceNative {
    ptr: ::winrt::ComPtr<IDesktopWindowXamlSourceNative>,
}

impl IDesktopWindowXamlSourceNative {
    pub fn attach_to_window(&self, hwnd: *mut core::ffi::c_void) -> winrt::Result<()> {
        // let this = winrt::NonNullRawComPtr::new(self.ptr).as_raw();
        // let this = self.ptr.get_abi().unwrap().as_raw();
        // (*this).vtable().attach_to_window(this, hwnd);
        
        let this = self
            .get_abi()
            .expect("IDesktopWindowXamlSourceNative not correctly initialized. Found null pointer.");

        unsafe {
            return (this.vtable().attach_to_window)(this, hwnd).ok();
        }
    }

    pub fn get_window_handle(&self) -> winrt::Result<*mut c_void> {
        let this = self
            .get_abi()
            .expect("IDesktopWindowXamlSourceNative not correctly initialized. Found null pointer.");

        let mut hwnd = ptr::null_mut();
        return (this.vtable().get_window_handle)(this, &mut hwnd).and_then(|| hwnd)
    }
}

unsafe impl ::winrt::ComInterface for IDesktopWindowXamlSourceNative {
    type VTable = abi_IDesktopWindowXamlSourceNative;
    fn iid() -> ::winrt::Guid {
        ::winrt::Guid::from_values(0x3cbcf1bf, 0x2f76, 0x4e9c, [0x96, 0xab, 0xe8, 0x4b, 0x37, 0x97, 0x25, 0x54])
    }
}

unsafe impl ::winrt::AbiTransferable for IDesktopWindowXamlSourceNative {
    type Abi = winrt::RawComPtr<Self>;

    fn get_abi(&self) -> Self::Abi {
        self.ptr.get_abi()
    }

    fn set_abi(&mut self) -> *mut Self::Abi {
        self.ptr.set_abi()
    }
}

#[repr(C)]
pub struct abi_IDesktopWindowXamlSourceNative {
    // pub unknown_query_interface: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>, &::winrt::Guid, *mut ::winrt::RawPtr) -> ::winrt::ErrorCode,
    // pub unknown_add_ref: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>) -> u32,
    // pub unknown_release: extern "system" fn(::winrt::RawComPtr<::winrt::IUnknown>) -> u32,
    __base: [usize; 3], // leave 3 ptr spaces empty for the IUnknown
    pub attach_to_window: extern "system" fn(winrt::NonNullRawComPtr<IDesktopWindowXamlSourceNative>, *mut c_void) -> ::winrt::ErrorCode,
    pub get_window_handle: extern "system" fn(winrt::NonNullRawComPtr<IDesktopWindowXamlSourceNative>, *mut *mut c_void) -> ::winrt::ErrorCode,
}

impl From<&DesktopWindowXamlSource> for IDesktopWindowXamlSourceNative {
    fn from(value: &DesktopWindowXamlSource) -> IDesktopWindowXamlSourceNative {
        <DesktopWindowXamlSource as ::winrt::ComInterface>::query(value)
    }
}

impl From<DesktopWindowXamlSource> for IDesktopWindowXamlSourceNative {
    fn from(value: DesktopWindowXamlSource) -> IDesktopWindowXamlSourceNative {
        std::convert::From::from(&value)
    }
}