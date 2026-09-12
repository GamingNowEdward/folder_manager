use std::ffi::c_void;

#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: isize,
        dw_attribute: u32,
        pv_attribute: *const c_void,
        cb_attribute: u32,
    ) -> i32;
}

const DWMWA_SYSTEMBACKDROP_TYPE: u32 = 38;
const DWMSBT_ACRYLIC: i32 = 3;

pub fn enable(hwnd: isize) -> bool {
    unsafe {
        let backdrop = DWMSBT_ACRYLIC;
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop as *const _ as *const c_void,
            std::mem::size_of::<i32>() as u32,
        ) == 0
    }
}
