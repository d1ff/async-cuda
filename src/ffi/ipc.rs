use cpp::cpp;

use crate::ffi::result;
use crate::ffi::ptr::DevicePtr;

type Result<T> = std::result::Result<T, crate::error::Error>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IpcMemHandle {
    pub reserved: [::std::ffi::c_char; 64usize],
}

impl Default for IpcMemHandle {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl IpcMemHandle {

    pub unsafe fn from_device_ptr(ptr: &DevicePtr) -> Result<Self> {
        let mut handle = IpcMemHandle::default();
        let ptr = ptr.as_ptr();
        let handle_ptr = std::ptr::addr_of_mut!(handle);

        let ret = cpp!(unsafe [
            handle_ptr as "void*",
            ptr as "void*"
        ] -> i32 as "std::int32_t" {
            return cudaIpcGetMemHandle((cudaIpcMemHandle_t*) handle_ptr, ptr);
        });

        result!(ret, handle)
    }

    pub unsafe fn get_device_ptr(self) -> Result<DevicePtr> {
        let mut d_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let d_ptr_ptr = std::ptr::addr_of_mut!(d_ptr);
        let handle = self;

        let ret = cpp!(unsafe [
            d_ptr_ptr as "void**",
            handle as "cudaIpcMemHandle_t"
        ] -> i32 as "std::int32_t" {
            return cudaIpcOpenMemHandle(d_ptr_ptr,
                handle,
                0);
        });

        result!(ret, DevicePtr::from_addr(d_ptr))
    }

    pub unsafe fn close(d: &mut DevicePtr) -> Result<()> {
        let mut d_ptr = d.take();

        let ret = cpp!(unsafe [
            d_ptr as "void*"
        ] -> i32 as "std::int32_t" {
            return cudaIpcCloseMemHandle(d_ptr);
        });

        result!(ret)
    }
    
}
