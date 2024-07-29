use nix::libc::size_t;
use std::ffi::{c_char, c_int, c_void};

pub mod pid;
pub mod pipe;
pub mod pty;
pub mod shm;

// #[repr(C)]
// pub struct Ipc {
//     receivable_fd: c_int,
//     sendable_fd: c_int,
//     shared_mem_fd: c_int,
//     increment: size_t,
//     shared_mem: c_void,
// }

#[repr(C)]
pub struct Ipc {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "ipc")]
extern "C" {
    pub fn ipc_open(handle: *const c_char, message_size: size_t) -> Ipc;
    pub fn ipc_send(ipc: *mut Ipc, message: *const u8) -> Ipc;
}
