use std::ffi::c_char;

use nix::libc::size_t;

#[repr(C)]
pub struct ipc_t {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "ipc")]
extern "C" {
    pub fn ipc_open(handle: *const c_char) -> *const ipc_t;
    pub fn ipc_send(ipc: *mut ipc_t, msg: *const u8, size: size_t);
    pub fn ipc_recv(ipc: *mut ipc_t, msg: *mut u8, size: size_t);
    pub fn ipc_destroy(ipc: *mut ipc_t);
}
