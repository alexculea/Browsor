use std::ffi::c_void;
use winrt::{AbiTransferable, ComInterface, ComPtr};

#[repr(C)]
pub struct abi_IInitializeWithWindow {
    __base: [usize; 3],
    initialize: extern "system" fn(
        winrt::NonNullRawComPtr<InitializeWithWindowInterop>,
        *mut c_void,
    ) -> winrt::ErrorCode,
}

#[repr(transparent)]
#[derive(Default, Clone)]
pub struct InitializeWithWindowInterop {
    ptr: ComPtr<InitializeWithWindowInterop>,
}

unsafe impl ComInterface for InitializeWithWindowInterop {
    type VTable = abi_IInitializeWithWindow;

    fn iid() -> winrt::Guid {
        winrt::Guid::from_values(1047057597, 28981, 19728, [128, 24, 159, 182, 217, 243, 63, 161])
    }
}

unsafe impl AbiTransferable for InitializeWithWindowInterop {
    type Abi = winrt::RawComPtr<Self>;

    fn get_abi(&self) -> Self::Abi {
        self.ptr.get_abi()
    }

    fn set_abi(&mut self) -> *mut Self::Abi {
        self.ptr.set_abi()
    }
}


impl InitializeWithWindowInterop {
    pub fn initialize(
        &self,
        hwnd: *mut c_void,
    ) -> winrt::Result<()> {

        let this = self
        .get_abi()
        .expect("InitializeWithWindowInterop not correctly initialized. Found null pointer.");

        unsafe {
            return (this.vtable().initialize)(this, hwnd).ok();
        }
    }
}