use std::{
    ffi::{OsString, c_int},
    iter::Once,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
};

type DWARD = u32;
type LPCWSTR = *const u16;

#[link(name = "Everything64")]
unsafe extern "C" {
    fn Everything_QueryW(bWait: c_int);
    fn Everything_SetSearchW(lpString: LPCWSTR);
    fn Everything_SetRequestFlags(dwRequestFlags: DWARD);
    fn Everything_SetMatchPath(bEnable: c_int);
    fn Everything_GetNumResults() -> DWARD;
    fn Everything_GetResultPathW(index: DWARD) -> LPCWSTR;
    fn Everything_GetResultFileNameW(index: DWARD) -> LPCWSTR;
}

fn string_to_u16s(str: String) -> Vec<u16> {
    let mut out: Vec<u16> = str.encode_utf16().collect();
    out.push(0);
    out
}

fn LPCWSTRs_to_pathbuf(path_ptr: LPCWSTR, fname_ptr: LPCWSTR) -> PathBuf {
    let mut len = 0;
    let path = unsafe {
        while *path_ptr.add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(path_ptr, len)
    };
    let path = OsString::from_wide(path);

    let mut len = 0;
    let fname = unsafe {
        while *fname_ptr.add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(fname_ptr, len)
    };
    let fname = OsString::from_wide(fname);

    let mut full: PathBuf = path.into();
    full.push(fname);
    full
}

pub fn search(search_str: String) -> Vec<PathBuf> {
    let mut out = Vec::new();

    let search_str = string_to_u16s(search_str);
    let search_lpstr = search_str.as_ptr();
    unsafe {
        Everything_SetRequestFlags(0x00000007);
        Everything_SetMatchPath(1);
        Everything_SetSearchW(search_lpstr);

        Everything_QueryW(1);
    };

    let result_len = unsafe { Everything_GetNumResults() };

    for i in 0..result_len {
        let path_lpstr = unsafe { Everything_GetResultPathW(i) };
        let fname_lpstr = unsafe { Everything_GetResultFileNameW(i) };
        out.push(LPCWSTRs_to_pathbuf(path_lpstr,fname_lpstr));
    }

    out
}
