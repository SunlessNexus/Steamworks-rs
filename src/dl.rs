#[allow(dead_code, unused)]
const RTLD_NOW: i32 = 2;

#[cfg(unix)]
#[allow(dead_code, unused)]
#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(filename: *const i8, flags: i32) -> *mut std::ffi::c_void;
    fn dlsym(handle: *mut std::ffi::c_void, symbol: *const u8) -> *mut std::ffi::c_void;
    fn dlerror() -> *const i8;
}

#[cfg(windows)]
#[allow(dead_code, unused)]
unsafe extern "system" {
    // https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibrarya
    // HMODULE LoadLibraryA(
    //    [in] LPCSTR lpLibFileName
    // );
    fn LoadLibraryA(lpLibFileName: *const i8) -> *mut std::ffi::c_void;
    // https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress
    // FARPROC GetProcAddress(
    //	[in] HMODULE hModule,
    //	[in] LPCSTR  lpProcName
    //	);
    fn GetProcAddress(hModule: *mut std::ffi::c_void, lpProcName: *const u8) -> *mut std::ffi::c_void;
    // https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-freelibrary
    // BOOL FreeLibrary(
    // [in] HMODULE hLibModule
    // );
    fn FreeLibrary(hLibModule: *mut std::ffi::c_void) -> bool;
}

pub struct Handle {
    pub module: *mut std::ffi::c_void
}

pub fn open(path: String) -> Option<Handle> {
    let path = match std::ffi::CString::new(path) {
        Ok(path) => path,
        Err(_) => return None
    };

    #[cfg(windows)]
    let module = unsafe {
        LoadLibraryA(path.as_ptr())
    };

    #[cfg(unix)]
    let module = unsafe {
        dlopen(path.as_ptr())
    };

    if module as usize == 0x0 {
        return None;
    }

    return Some(Handle { 
        module: module
    })
}

impl Handle {
    pub fn find(&self, name: &[u8]) -> *mut std::ffi::c_void {
        #[cfg(windows)]
        unsafe {
            GetProcAddress(self.module, name.as_ptr())
        }

        #[cfg(unix)]
        unsafe {
            dlsym(self.module, name.as_ptr())
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            FreeLibrary(self.module);
        }

        #[cfg(unix)]
        unsafe {
            dlclose(self.module);
        }
    }
}