mod utils;
use std::path::Path;

use crate::si2;
use crate::{Lef, LefResult};

impl Lef {
    pub fn load<P: AsRef<Path>>(path: P) -> LefResult<Self> {
        unsafe { Self::load_inner(path.as_ref()) }
    }

    unsafe fn load_inner(path: &Path) -> LefResult<Self> {
        let path = path.to_str().unwrap();
        let mut lef = Lef::default();
        
        unsafe { 
            si2::lefrInit(); 
            si2::lefrSetVersionCbk(Some(Self::set_version));

            let self_ptr = &mut lef as *mut Lef as *mut ::std::os::raw::c_void;

            let (fp, fp_for_lefr) = utils::open_c_file(path, "r");
            let ret = si2::lefrRead(fp_for_lefr, path.as_ptr() as *const std::os::raw::c_char, self_ptr);
            assert!(ret == 0);
        
            si2::lefrReleaseNResetMemory();

            libc::fclose(fp)
        };

        Ok(lef)
    }

    unsafe extern "C" fn set_version(
        _: si2::lefrCallbackType_e,
        version: f64,
        ud: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        unsafe {
            let lef = &mut *(ud as *mut Lef);
            lef.version = Some(version);
        }
        0
    }
}

