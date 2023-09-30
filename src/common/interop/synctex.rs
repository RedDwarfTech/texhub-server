use std::os::raw::{c_int, c_char};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct synctex_scanner_t {
    _unused: [u8; 0],
}
pub type synctex_scanner_s = synctex_scanner_t;
pub type synctex_scanner_p = *mut synctex_scanner_s;

// https://stackoverflow.com/questions/77206184/usr-bin-ld-cannot-find-lsynctex-parser-no-such-file-or-directory
#[link(name = "synctex_parser")]
extern "C" {
    pub fn synctex_scanner_new_with_output_file(
        output: *const ::std::os::raw::c_char,
        build_directory: *const ::std::os::raw::c_char,
        parse: ::std::os::raw::c_int,
    ) -> synctex_scanner_p;
}
