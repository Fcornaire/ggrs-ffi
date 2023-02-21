use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_float, c_int};

pub fn char_c_array_to_vec_string(array_c: *mut *mut c_char) -> Vec<String> {
    let mut resultat = Vec::new();

    let mut i = 0;
    loop {
        let ptr = unsafe { *array_c.offset(i) };
        if ptr.is_null() {
            break;
        }

        let c_str = unsafe { CStr::from_ptr(ptr) };
        let rust_str = c_str.to_string_lossy().into_owned();

        resultat.push(rust_str);

        i += 1;
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
