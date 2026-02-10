//! Provides byte buffer utilities to send bytes across FFI.
//! As byte buffer boxed bytes slice is used `Box<[u8]>`

use std::{
    alloc::{Layout, alloc_zeroed},
    mem::ManuallyDrop,
};

/// Allocates a new zeroed byte buffer (layout `Box<[u8]>`) with the given `length`
/// and returns the pointer to the buffer.
///
/// The returned buffer will not be dropped - lifetime is not rust managed,
/// so the buffer can be passed to the FFI client or hosts to be filled.
///
/// # Safety
///
/// Later at some point, after the buffer is filled, the buffer must be converted
/// to rust managed boxed byte slice with one of the `from_...` functions.
pub fn new_boxed_byte_slice_buffer_raw(length: usize) -> *mut u8 {
    if length == 0 {
        return std::ptr::null_mut();
    }

    // Basically the same as 'vec![0; 512].into_boxed_slice()', but with less conversion steps
    // involved and no 'ManuallyDrop' needed.

    let layout = Layout::array::<u8>(length).unwrap_or_else(|_| panic!("capacity overflow"));
    unsafe { alloc_zeroed(layout) }
}

pub fn string_into_boxed_byte_slice_raw(src: String) -> (*const u8, usize) {
    if src.is_empty() {
        return (std::ptr::null(), 0);
    }

    let slice: Box<[u8]> = Box::from(src.as_str().as_bytes());

    into_boxed_byte_slice_raw(slice)
}

pub fn into_boxed_byte_slice_raw(src: Box<[u8]>) -> (*const u8, usize) {
    if src.is_empty() {
        return (std::ptr::null(), 0);
    }

    let len = src.len();
    let ptr = src.as_ptr();

    let _ = ManuallyDrop::new(src);

    (ptr, len)
}

pub fn from_boxed_byte_slice_raw(slice_ptr: *mut u8, length: usize) -> Box<[u8]> {
    if length == 0 {
        return Box::default();
    }

    let slice_raw = std::ptr::slice_from_raw_parts_mut(slice_ptr, length);
    unsafe { Box::from_raw(slice_raw) }
}

// `trim` - if true leading and trailing whitespace will be removed.
pub fn string_from_boxed_byte_slice_raw(slice_ptr: *mut u8, length: usize, trim: bool) -> String {
    if length == 0 {
        return String::default();
    }

    let slice = from_boxed_byte_slice_raw(slice_ptr, length);
    let str = unsafe { std::str::from_boxed_utf8_unchecked(slice) };

    if trim {
        return str.trim().to_string();
    }

    str.to_string()
}

/*pub fn vec_from_boxed_byte_slice_raw(slice_ptr: *mut u8, length: usize) -> Vec<u8> {
    from_boxed_byte_slice_raw(slice_ptr, length).to_vec()
}*/

/*// Returns a rust byte slice representation of the given
/// C-Bytes, received and owned from C.
///
/// # Arguments
/// - `c_bytes_ptr` - pointer to the C-Bytes
/// - `c_bytes_len` - length of the C-Bytes
///
/// # Safety
///
/// The given C-Bytes must be valid (not deallocated from the owning C side)
/// while the returned reference is used.
///
/// Note: The given C-Bytes are not deallocated or dropped in any form, that must be
/// done by the owning C side.
pub const unsafe fn c_bytes_as_slice_ref<'a>(
    c_bytes_ptr: *const u8,
    c_bytes_len: usize,
) -> &'a [u8] {
    slice::from_raw_parts(c_bytes_ptr, c_bytes_len)
}

/// Returns a rust string slice representation of the given
/// C-Bytes, received and owned from C.
///
/// # Arguments
/// - `c_bytes_ptr` - pointer to the C-Bytes
/// - `c_bytes_len` - length of the C-Bytes
///
/// # Safety
///
/// The given C-Bytes must be valid (not deallocated from the owning C side)
/// while the returned reference is used and the bytes must be valid UTF-8.
///
/// Note: The given C-Bytes are not deallocated or dropped in any form, that must be
/// done by the owning C side.
pub const unsafe fn c_bytes_as_str_ref<'a>(c_bytes_ptr: *const u8, c_bytes_len: usize) -> &'a str {
    from_utf8_unchecked(c_bytes_as_slice_ref(c_bytes_ptr, c_bytes_len))
}

/// Returns a new rust string from the given C-Bytes, received and owned from C.
///
/// # Arguments
/// - `c_bytes_ptr` - pointer to the C-Bytes
/// - `c_bytes_len` - length of the C-Bytes
///
/// # Safety
///
/// The given C-Bytes must be valid (not deallocated from the owning C side)
/// while this function is in process of creating the rust string and the bytes must be valid UTF-8.
///
/// Note: The given C-Bytes are not deallocated or dropped in any form, that must be
/// done by the owning C side.
pub unsafe fn c_bytes_to_string(c_bytes_ptr: *const u8, c_bytes_len: usize) -> String {
    c_bytes_as_str_ref(c_bytes_ptr, c_bytes_len).to_string()
}

/// Returns a new `[u8; 6]` byte array from the given C-Bytes, received and owned from C.
///
/// Use cases are where mac address bytes (length of 6) are received from C, like:
/// - `BTAddress` (Android - mac address type)
/// - `BTSerial`
///
/// # Arguments
/// - `c_bytes_ptr` - pointer to the C-Bytes
/// - `c_bytes_len` - length of the C-Bytes
///
/// # Panics
///
/// This function will panic if the given C-Bytes have not at least a length of 6.
/// Further bytes beyond 6 will be ignored if present.
///
/// # Safety
///
/// The given C-Bytes must be valid (not deallocated from the owning C side)
/// while this function is in process of creating the rust array.
///
/// Note: The given C-Bytes are not deallocated or dropped in any form, that must be
/// done by the owning C side.
pub const unsafe fn c_bytes_to_6_bytes_cap_array(
    c_bytes_ptr: *const u8,
    c_bytes_len: usize,
) -> [u8; 6] {
    let b = c_bytes_as_slice_ref(c_bytes_ptr, c_bytes_len);
    [b[0], b[1], b[2], b[3], b[4], b[5]]
}

/// Returns a new `[u8; 16]` byte array from the given C-Bytes, received and owned from C.
///
/// Use cases are where uuid bytes (length of 16) are received from C, like:
/// - `BTAddress` (IOS - uuid type)
/// - `BTUuid`
///
/// # Arguments
/// - `c_bytes_ptr` - pointer to the C-Bytes
/// - `c_bytes_len` - length of the C-Bytes
///
/// # Panics
///
/// This function will panic if the given C-Bytes have not at least a length of 16.
/// Further bytes beyond 16 will be ignored if present.
///
/// # Safety
///
/// The given C-Bytes must be valid (not deallocated from the owning C side)
/// while this function is in process of creating the rust array.
///
/// Note: The given C-Bytes are not deallocated or dropped in any form, that must be
/// done by the owning C side.
pub const unsafe fn c_bytes_to_16_bytes_cap_array(
    c_bytes_ptr: *const u8,
    c_bytes_len: usize,
) -> [u8; 16] {
    let b = c_bytes_as_slice_ref(c_bytes_ptr, c_bytes_len);
    [
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13],
        b[14], b[15],
    ]
}*/
