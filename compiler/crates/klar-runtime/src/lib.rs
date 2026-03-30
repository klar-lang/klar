//! Klar Runtime Library
//!
//! Provides ARC (Automatic Reference Counting) memory management,
//! string operations, list/map/set collections, and other runtime
//! support for natively compiled Klar programs.

use std::alloc::{Layout, alloc, dealloc};
use std::ptr;
use std::ffi::CStr;

// ============================================================
// ARC (Automatic Reference Counting)
// ============================================================

/// Header for reference-counted objects.
/// Placed before every heap-allocated Klar value.
#[repr(C)]
struct ArcHeader {
    ref_count: u64,
    size: u64,       // Size of the payload in bytes
    type_tag: u64,   // Type discriminant for cycle detection
}

const HEADER_SIZE: usize = std::mem::size_of::<ArcHeader>();

/// Allocate a new ARC-managed object with the given payload size.
/// Returns a pointer to the payload (past the header).
#[unsafe(no_mangle)]
pub extern "C" fn klar_arc_alloc(size: u64, type_tag: u64) -> *mut u8 {
    let total_size = HEADER_SIZE + size as usize;
    let layout = Layout::from_size_align(total_size, 8).unwrap();
    unsafe {
        let ptr = alloc(layout);
        if ptr.is_null() {
            std::process::abort();
        }
        let header = ptr as *mut ArcHeader;
        (*header).ref_count = 1;
        (*header).size = size;
        (*header).type_tag = type_tag;
        ptr.add(HEADER_SIZE)
    }
}

/// Increment the reference count.
#[unsafe(no_mangle)]
pub extern "C" fn klar_arc_retain(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let header = (ptr as *mut ArcHeader).offset(-1);
        (*header).ref_count += 1;
    }
}

/// Decrement the reference count. Frees if it reaches zero.
#[unsafe(no_mangle)]
pub extern "C" fn klar_arc_release(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let header = (ptr as *mut ArcHeader).offset(-1);
        (*header).ref_count -= 1;
        if (*header).ref_count == 0 {
            let total_size = HEADER_SIZE + (*header).size as usize;
            let layout = Layout::from_size_align(total_size, 8).unwrap();
            dealloc(header as *mut u8, layout);
        }
    }
}

/// Get the current reference count (for debugging).
#[unsafe(no_mangle)]
pub extern "C" fn klar_arc_count(ptr: *mut u8) -> u64 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let header = (ptr as *mut ArcHeader).offset(-1);
        (*header).ref_count
    }
}

// ============================================================
// Strings
// ============================================================

/// Klar String representation: ARC-managed, pointer + length.
#[repr(C)]
pub struct KlarString {
    data: *const u8,
    len: u64,
}

/// Create a new Klar string from a C string literal.
#[unsafe(no_mangle)]
pub extern "C" fn klar_string_from_cstr(cstr: *const i8) -> *mut KlarString {
    unsafe {
        let len = libc_strlen(cstr as *const u8);
        let str_ptr = klar_arc_alloc(std::mem::size_of::<KlarString>() as u64, 1) as *mut KlarString;
        let data = klar_arc_alloc(len as u64 + 1, 2);
        ptr::copy_nonoverlapping(cstr as *const u8, data, len as usize + 1);
        (*str_ptr).data = data;
        (*str_ptr).len = len as u64;
        str_ptr
    }
}

/// Get the length of a Klar string.
#[unsafe(no_mangle)]
pub extern "C" fn klar_string_len(s: *const KlarString) -> u64 {
    if s.is_null() {
        return 0;
    }
    unsafe { (*s).len }
}

/// Get the C string pointer from a Klar string.
#[unsafe(no_mangle)]
pub extern "C" fn klar_string_cstr(s: *const KlarString) -> *const u8 {
    if s.is_null() {
        return b"\0".as_ptr();
    }
    unsafe { (*s).data }
}

/// Concatenate two Klar strings.
#[unsafe(no_mangle)]
pub extern "C" fn klar_string_concat(a: *const KlarString, b: *const KlarString) -> *mut KlarString {
    unsafe {
        let a_len = if a.is_null() { 0 } else { (*a).len as usize };
        let b_len = if b.is_null() { 0 } else { (*b).len as usize };
        let total = a_len + b_len;

        let str_ptr = klar_arc_alloc(std::mem::size_of::<KlarString>() as u64, 1) as *mut KlarString;
        let data = klar_arc_alloc(total as u64 + 1, 2);

        if a_len > 0 {
            ptr::copy_nonoverlapping((*a).data, data, a_len);
        }
        if b_len > 0 {
            ptr::copy_nonoverlapping((*b).data, data.add(a_len), b_len);
        }
        *data.add(total) = 0; // null terminator

        (*str_ptr).data = data;
        (*str_ptr).len = total as u64;
        str_ptr
    }
}

// ============================================================
// Lists
// ============================================================

/// Klar List representation: ARC-managed dynamic array.
#[repr(C)]
pub struct KlarList {
    data: *mut u8,        // Pointer to array of elements
    len: u64,             // Number of elements
    cap: u64,             // Capacity
    elem_size: u64,       // Size of each element in bytes
}

/// Create a new empty list with the given element size.
#[unsafe(no_mangle)]
pub extern "C" fn klar_list_new(elem_size: u64) -> *mut KlarList {
    let list_ptr = klar_arc_alloc(std::mem::size_of::<KlarList>() as u64, 3) as *mut KlarList;
    unsafe {
        let initial_cap = 8u64;
        let data = klar_arc_alloc(initial_cap * elem_size, 4);
        (*list_ptr).data = data;
        (*list_ptr).len = 0;
        (*list_ptr).cap = initial_cap;
        (*list_ptr).elem_size = elem_size;
    }
    list_ptr
}

/// Push an element to the end of the list.
#[unsafe(no_mangle)]
pub extern "C" fn klar_list_push(list: *mut KlarList, elem: *const u8) {
    unsafe {
        if (*list).len >= (*list).cap {
            // Grow: double capacity
            let new_cap = (*list).cap * 2;
            let new_data = klar_arc_alloc(new_cap * (*list).elem_size, 4);
            let old_size = (*list).len * (*list).elem_size;
            ptr::copy_nonoverlapping((*list).data, new_data, old_size as usize);
            klar_arc_release((*list).data);
            (*list).data = new_data;
            (*list).cap = new_cap;
        }
        let offset = ((*list).len * (*list).elem_size) as usize;
        ptr::copy_nonoverlapping(elem, (*list).data.add(offset), (*list).elem_size as usize);
        (*list).len += 1;
    }
}

/// Get a pointer to the element at the given index.
#[unsafe(no_mangle)]
pub extern "C" fn klar_list_get(list: *const KlarList, index: u64) -> *const u8 {
    unsafe {
        if index >= (*list).len {
            eprintln!("index out of bounds: {} >= {}", index, (*list).len);
            std::process::abort();
        }
        let offset = (index * (*list).elem_size) as usize;
        (*list).data.add(offset)
    }
}

/// Get the length of the list.
#[unsafe(no_mangle)]
pub extern "C" fn klar_list_len(list: *const KlarList) -> u64 {
    if list.is_null() {
        return 0;
    }
    unsafe { (*list).len }
}

// ============================================================
// Maps
// ============================================================

/// Simple map using linear probing (will be upgraded to hash map).
#[repr(C)]
pub struct KlarMap {
    keys: *mut u8,
    values: *mut u8,
    len: u64,
    cap: u64,
    key_size: u64,
    val_size: u64,
}

/// Create a new empty map.
#[unsafe(no_mangle)]
pub extern "C" fn klar_map_new(key_size: u64, val_size: u64) -> *mut KlarMap {
    let map_ptr = klar_arc_alloc(std::mem::size_of::<KlarMap>() as u64, 5) as *mut KlarMap;
    unsafe {
        let initial_cap = 16u64;
        (*map_ptr).keys = klar_arc_alloc(initial_cap * key_size, 6);
        (*map_ptr).values = klar_arc_alloc(initial_cap * val_size, 7);
        (*map_ptr).len = 0;
        (*map_ptr).cap = initial_cap;
        (*map_ptr).key_size = key_size;
        (*map_ptr).val_size = val_size;
    }
    map_ptr
}

/// Get the number of entries in the map.
#[unsafe(no_mangle)]
pub extern "C" fn klar_map_len(map: *const KlarMap) -> u64 {
    if map.is_null() {
        return 0;
    }
    unsafe { (*map).len }
}

// ============================================================
// Sets
// ============================================================

/// Create a new empty set.
#[unsafe(no_mangle)]
pub extern "C" fn klar_set_new(elem_size: u64) -> *mut KlarList {
    klar_list_new(elem_size)
}

// ============================================================
// Printing helpers
// ============================================================

/// Print an integer followed by newline.
#[unsafe(no_mangle)]
pub extern "C" fn klar_print_int(n: i64) {
    println!("{}", n);
}

/// Print a float followed by newline.
#[unsafe(no_mangle)]
pub extern "C" fn klar_print_float(n: f64) {
    println!("{}", n);
}

/// Print a boolean followed by newline.
#[unsafe(no_mangle)]
pub extern "C" fn klar_print_bool(b: bool) {
    println!("{}", b);
}

/// Print a C string followed by newline.
#[unsafe(no_mangle)]
pub extern "C" fn klar_print_str(s: *const u8) {
    if s.is_null() {
        println!("(null)");
        return;
    }
    unsafe {
        let cstr = CStr::from_ptr(s as *const i8);
        println!("{}", cstr.to_string_lossy());
    }
}

/// Assert that a condition is true.
#[unsafe(no_mangle)]
pub extern "C" fn klar_assert(cond: bool, msg: *const u8) {
    if !cond {
        if !msg.is_null() {
            unsafe {
                let cstr = CStr::from_ptr(msg as *const i8);
                eprintln!("assertion failed: {}", cstr.to_string_lossy());
            }
        } else {
            eprintln!("assertion failed");
        }
        std::process::exit(1);
    }
}

// ============================================================
// Int-to-string conversion for string interpolation
// ============================================================

/// Convert an integer to a string (for interpolation).
#[unsafe(no_mangle)]
pub extern "C" fn klar_int_to_str(n: i64) -> *mut u8 {
    let s = format!("{}\0", n);
    let ptr = klar_arc_alloc(s.len() as u64, 2);
    unsafe {
        ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len());
    }
    ptr
}

/// Convert a float to a string (for interpolation).
#[unsafe(no_mangle)]
pub extern "C" fn klar_float_to_str(n: f64) -> *mut u8 {
    let s = format!("{}\0", n);
    let ptr = klar_arc_alloc(s.len() as u64, 2);
    unsafe {
        ptr::copy_nonoverlapping(s.as_ptr(), ptr, s.len());
    }
    ptr
}

// ============================================================
// Internal helpers
// ============================================================

unsafe fn libc_strlen(s: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}
