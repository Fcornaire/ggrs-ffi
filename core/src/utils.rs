use core::slice;
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_float, c_int};

use libc::size_t;
use uuid::Uuid;

pub fn char_c_array_to_vec_string(array_c: *mut *mut c_char, len: size_t) -> Vec<String> {
    let mut resultat = Vec::new();

    for i in 0..len {
        let ptr = unsafe { *array_c.offset(i as isize) };
        if ptr.is_null() {
            break;
        }

        let c_str = unsafe { CStr::from_ptr(ptr) };
        let rust_str = c_str.to_string_lossy().into_owned();

        resultat.push(rust_str);
    }

    resultat
}

pub fn copy_vec_string_to_char_c_array(v_string: &Vec<String>, array_c: *mut *mut c_char) {
    for (i, str) in v_string.iter().enumerate() {
        let str = CString::new(str.as_bytes()).unwrap().into_raw();

        unsafe { libc::strcpy(*array_c.offset(i as isize), str) };
    }
}

fn copy_int_to_array_c(int: i32, array_c: *mut c_int, index: usize) {
    unsafe {
        *array_c.offset(index as isize) = mem::transmute(int);
    }
}

pub fn copy_vec_int_to_int_array_c(vec_int: &Vec<i32>, array_c: *mut c_int) {
    for (i, int) in vec_int.iter().enumerate() {
        copy_int_to_array_c(*int, array_c, i);
    }
}

fn copy_float_to_array_c(float: f32, array_c: *mut c_float, index: usize) {
    unsafe {
        *array_c.offset(index as isize) = mem::transmute(float);
    }
}

pub fn copy_vec_float_to_float_array_c(vec_float: &Vec<f32>, array_c: *mut c_float) {
    for (i, float) in vec_float.iter().enumerate() {
        copy_float_to_array_c(*float, array_c, i);
    }
}

pub fn byte_array_to_guid(byte_array_ptr: *mut u8) -> Uuid {
    let guid_vec = unsafe { slice::from_raw_parts(byte_array_ptr, 16) };
    let uuid = Uuid::from_slice(&guid_vec).unwrap();

    uuid
}

pub fn string_guid_to_byte_array(guid: String) -> [u8; 16] {
    Uuid::parse_str(&guid.to_string())
        .unwrap()
        .as_bytes()
        .clone()
}
