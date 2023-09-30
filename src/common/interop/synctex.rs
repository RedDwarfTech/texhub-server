use std::os::raw::{c_int, c_char};

#[link(name = "synctex_parser")]
extern "C" {
    pub fn synctex_scanner_new_with_output_file(output: c_char, build_directory: c_char, parse: c_int);
}
