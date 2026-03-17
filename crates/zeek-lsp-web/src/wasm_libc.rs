/// C stdlib shims for wasm32-unknown-unknown.
/// These prevent tree-sitter's C code from generating "env" WASM imports,
/// which browsers cannot satisfy without an importmap polyfill.
use core::ffi::c_void;
use std::alloc::Layout;

// Each allocation is prefixed with an 8-byte header storing the usable size.
const HEADER: usize = 8;

unsafe fn alloc_with_header(size: usize) -> *mut c_void {
    if size == 0 {
        return core::ptr::null_mut();
    }
    let layout = Layout::from_size_align_unchecked(size + HEADER, HEADER);
    let raw = std::alloc::alloc(layout);
    if raw.is_null() {
        return core::ptr::null_mut();
    }
    (raw as *mut usize).write(size);
    raw.add(HEADER) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    alloc_with_header(size)
}

#[no_mangle]
pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    let total = nmemb.saturating_mul(size);
    let ptr = alloc_with_header(total);
    if !ptr.is_null() {
        core::ptr::write_bytes(ptr as *mut u8, 0, total);
    }
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, new_size: usize) -> *mut c_void {
    if ptr.is_null() {
        return alloc_with_header(new_size);
    }
    if new_size == 0 {
        free(ptr);
        return core::ptr::null_mut();
    }
    let raw = (ptr as *mut u8).sub(HEADER);
    let old_size = (raw as *const usize).read();
    let old_layout = Layout::from_size_align_unchecked(old_size + HEADER, HEADER);
    let new_raw = std::alloc::realloc(raw, old_layout, new_size + HEADER);
    if new_raw.is_null() {
        return core::ptr::null_mut();
    }
    (new_raw as *mut usize).write(new_size);
    new_raw.add(HEADER) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let raw = (ptr as *mut u8).sub(HEADER);
    let size = (raw as *const usize).read();
    let layout = Layout::from_size_align_unchecked(size + HEADER, HEADER);
    std::alloc::dealloc(raw, layout);
}

#[no_mangle]
pub extern "C" fn abort() -> ! {
    panic!("abort() called from C")
}

#[no_mangle]
pub unsafe extern "C" fn __assert_fail(
    _msg: *const u8,
    _file: *const u8,
    _line: u32,
    _func: *const u8,
) {
    panic!("C assertion failed")
}

// Timestamp — not needed for parsing correctness.
#[no_mangle]
pub extern "C" fn now() -> f64 {
    0.0
}

// No-op I/O — tree-sitter only uses these for debug output.
// Signatures match the WASM import types produced by this binary.
#[no_mangle]
pub unsafe extern "C" fn fprintf(_stream: *mut c_void, _fmt: *const u8, _arg: usize) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fwrite(
    _ptr: *const c_void,
    _size: usize,
    _nmemb: usize,
    _stream: *mut c_void,
) -> usize {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fputc(_c: i32, _stream: *mut c_void) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fclose(_stream: *mut c_void) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fdopen(_fd: i32, _mode: *const u8) -> *mut c_void {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn snprintf(
    _buf: *mut u8,
    _n: usize,
    _fmt: *const u8,
    _arg: usize,
) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn vsnprintf(
    _buf: *mut u8,
    _n: usize,
    _fmt: *const u8,
    _args: *mut c_void,
) -> i32 {
    0
}

// strncmp — used by tree-sitter for keyword matching.
#[no_mangle]
pub unsafe extern "C" fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return (a as i32) - (b as i32);
        }
        if a == 0 {
            return 0;
        }
    }
    0
}

// Wide character classification — used by tree-sitter's lexer.
#[no_mangle]
pub extern "C" fn iswspace(c: i32) -> i32 {
    matches!(
        c as u32,
        0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x20 | 0x85 | 0xA0
            | 0x1680
            | 0x2000..=0x200A
            | 0x2028
            | 0x2029
            | 0x202F
            | 0x205F
            | 0x3000
    ) as i32
}

#[no_mangle]
pub extern "C" fn iswalnum(c: i32) -> i32 {
    let c = c as u32;
    ((c >= 0x30 && c <= 0x39) || (c >= 0x41 && c <= 0x5A) || (c >= 0x61 && c <= 0x7A) || c > 0x7F) as i32
}
