use serde::{Serialize, Deserialize};

use crate::{ffi::ipc::IpcMemHandle, DeviceBuffer, DeviceId};
use crate::ffi::memory::DeviceBuffer as DeviceBufferInternal;
use crate::runtime::Future;

type Result<T> = std::result::Result<T, crate::error::Error>;

#[derive(Serialize, Deserialize)]
pub struct IpcMemHandleSized<T: Copy> {
    pub num_elements: usize,
    internal: IpcMemHandle,
    device: DeviceId,
    _phantom: std::marker::PhantomData<T>,
}

unsafe impl<T: Copy> Send for IpcMemHandleSized<T> {}
unsafe impl<T: Copy> Sync for IpcMemHandleSized<T> {}

impl <T: Copy> IpcMemHandleSized<T> {

    pub async fn new_from_device_buffer(buffer: DeviceBuffer<T>) -> Result<Self> {
        Future::new(move || {
            let inner = buffer.inner();
            let d_ptr = inner.as_internal();
            let internal = unsafe { IpcMemHandle::from_device_ptr(d_ptr)? };
            Ok(Self {
                num_elements: inner.num_elements,
                internal,
                device: inner.device,
                _phantom: Default::default()
            })
        }).await
    }

}

pub struct IpcMemHandleGuard<'guard, T: Copy + 'static> {
    handle: &'guard IpcMemHandleSized<T>,
    buffer: DeviceBuffer<T>,
}

impl<T: Copy + 'static> Drop for IpcMemHandleGuard<'_, T> {

    fn drop(&mut self) {
        unsafe {
            // steal DevicePtr from this DeviceBuffer, so we can close the handle
            let mut d_ptr = self.buffer.inner_mut().as_mut_internal().take();

            IpcMemHandle::close(&mut d_ptr).unwrap();
        }
    }
    
}

impl <T: Copy + 'static> IpcMemHandleGuard<'_, T> {

    pub async fn new<'handle>(handle: &'handle IpcMemHandleSized<T>) -> Result<IpcMemHandleGuard<'handle, T>> {
        let buffer_internal = Future::new(move || {
            unsafe {
                let d_ptr = handle.internal.get_device_ptr()?;

                Ok(DeviceBufferInternal::from_num_elems_internal_device(
                    handle.num_elements, d_ptr, handle.device))
            }
        }).await?;

        Ok(IpcMemHandleGuard { 
            handle: handle, 
            buffer: unsafe { DeviceBuffer::from_internal(buffer_internal) }
        })
    }

    pub fn buffer(&self) -> &DeviceBuffer<T> {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut DeviceBuffer<T> {
        &mut self.buffer
    }
    
}

