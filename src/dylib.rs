// Cross-platform dynamic library loading module

use std::ffi::CString;
use std::ptr;

#[cfg(unix)]
use libc::{dlclose, dlerror, dlopen, dlsym, RTLD_LAZY};

#[cfg(windows)]
use windows_sys::Win32::Foundation::FreeLibrary;
#[cfg(windows)]
use windows_sys::Win32::System::LibraryLoader::{
    GetProcAddress, LoadLibraryA, GetModuleHandleA,
};

pub struct DynamicLibrary {
    #[cfg(unix)]
    handle: *mut libc::c_void,
    #[cfg(windows)]
    handle: isize, // HMODULE on Windows
}

impl DynamicLibrary {
    /// Load a dynamic library by name
    pub fn load(name: &str) -> Result<Self, String> {
        #[cfg(unix)]
        {
            let c_name = CString::new(name).map_err(|e| e.to_string())?;
            let handle = unsafe { dlopen(c_name.as_ptr(), RTLD_LAZY) };

            if handle.is_null() {
                let error_msg = unsafe {
                    let err_ptr = dlerror();
                    if !err_ptr.is_null() {
                        std::ffi::CStr::from_ptr(err_ptr)
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        format!("Failed to load library: {}", name)
                    }
                };
                return Err(error_msg);
            }

            Ok(Self { handle })
        }

        #[cfg(windows)]
        {
            let c_name = CString::new(name).map_err(|e| e.to_string())?;
            let handle = unsafe { LoadLibraryA(c_name.as_ptr() as *const u8) };

            if handle == 0 {
                return Err(format!("Failed to load library: {}", name));
            }

            Ok(Self { handle })
        }
    }

    /// Load the default C library / main program symbols
    pub fn load_default() -> Result<Self, String> {
        #[cfg(unix)]
        {
            // On Unix, passing NULL to dlopen loads the main program and its dependencies
            let handle = unsafe { dlopen(ptr::null(), RTLD_LAZY) };

            if handle.is_null() {
                return Err("Failed to load default C library".to_string());
            }

            Ok(Self { handle })
        }

        #[cfg(windows)]
        {
            // On Windows, get a handle to the msvcrt.dll (C runtime)
            let c_name = CString::new("msvcrt.dll").unwrap();
            let handle = unsafe { LoadLibraryA(c_name.as_ptr() as *const u8) };

            if handle == 0 {
                // Try to get the main module handle as fallback
                let main_handle = unsafe { GetModuleHandleA(ptr::null()) };
                if main_handle == 0 {
                    return Err("Failed to load default C library".to_string());
                }
                return Ok(Self {
                    handle: main_handle,
                });
            }

            Ok(Self { handle })
        }
    }

    /// Get a symbol from the library
    pub fn get_symbol(&self, name: &str) -> Option<*mut libc::c_void> {
        #[cfg(unix)]
        {
            let c_name = CString::new(name).ok()?;
            let sym = unsafe { dlsym(self.handle, c_name.as_ptr()) };

            if sym.is_null() {
                None
            } else {
                Some(sym)
            }
        }

        #[cfg(windows)]
        {
            let c_name = CString::new(name).ok()?;
            let sym = unsafe { GetProcAddress(self.handle, c_name.as_ptr() as *const u8) };

            if sym.is_none() {
                None
            } else {
                Some(sym.unwrap() as *mut libc::c_void)
            }
        }
    }
}

impl Drop for DynamicLibrary {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            dlclose(self.handle);
        }

        #[cfg(windows)]
        unsafe {
            // Note: We don't free the main module handle
            if self.handle != 0 {
                FreeLibrary(self.handle);
            }
        }
    }
}

// Ensure the type is Send and Sync for multi-threaded use
unsafe impl Send for DynamicLibrary {}
unsafe impl Sync for DynamicLibrary {}
