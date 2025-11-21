use std::ptr;

use mlua::prelude::*;

use crate::ctype::CType;
use crate::dylib::DynamicLibrary;

// Helper function to read a value from memory as a Lua value
#[inline]
fn read_ctype_value(lua: &Lua, ptr: *mut u8, ctype: &CType) -> LuaResult<LuaValue> {
    unsafe {
        match ctype {
            // Basic integer types
            CType::Int => Ok(LuaValue::Integer(*(ptr as *const i32) as i64)),
            CType::UInt => Ok(LuaValue::Integer(*(ptr as *const u32) as i64)),
            CType::Long => Ok(LuaValue::Integer(*(ptr as *const isize) as i64)),
            CType::ULong => Ok(LuaValue::Integer(*(ptr as *const usize) as i64)),
            CType::LongLong => Ok(LuaValue::Integer(*(ptr as *const i64))),
            CType::ULongLong => Ok(LuaValue::Integer(*(ptr as *const u64) as i64)),
            
            // Character types
            CType::Char => Ok(LuaValue::Integer(*(ptr as *const i8) as i64)),
            CType::UChar => Ok(LuaValue::Integer(*(ptr as *const u8) as i64)),
            
            // Short types
            CType::Short => Ok(LuaValue::Integer(*(ptr as *const i16) as i64)),
            CType::UShort => Ok(LuaValue::Integer(*(ptr as *const u16) as i64)),
            
            // Fixed-width integer types
            CType::Int8 => Ok(LuaValue::Integer(*(ptr as *const i8) as i64)),
            CType::Int16 => Ok(LuaValue::Integer(*(ptr as *const i16) as i64)),
            CType::Int32 => Ok(LuaValue::Integer(*(ptr as *const i32) as i64)),
            CType::Int64 => Ok(LuaValue::Integer(*(ptr as *const i64))),
            CType::UInt8 => Ok(LuaValue::Integer(*(ptr as *const u8) as i64)),
            CType::UInt16 => Ok(LuaValue::Integer(*(ptr as *const u16) as i64)),
            CType::UInt32 => Ok(LuaValue::Integer(*(ptr as *const u32) as i64)),
            CType::UInt64 => Ok(LuaValue::Integer(*(ptr as *const u64) as i64)),
            
            // Size types
            CType::SizeT => Ok(LuaValue::Integer(*(ptr as *const usize) as i64)),
            CType::SSizeT => Ok(LuaValue::Integer(*(ptr as *const isize) as i64)),
            
            // Floating point types
            CType::Float => Ok(LuaValue::Number(*(ptr as *const f32) as f64)),
            CType::Double => Ok(LuaValue::Number(*(ptr as *const f64))),
            
            // Boolean type
            CType::Bool => Ok(LuaValue::Boolean(*(ptr as *const bool))),
            
            // POSIX types (Unix only)
            #[cfg(unix)]
            CType::InoT => Ok(LuaValue::Integer(*(ptr as *const libc::ino_t) as i64)),
            #[cfg(unix)]
            CType::DevT => Ok(LuaValue::Integer(*(ptr as *const libc::dev_t) as i64)),
            #[cfg(unix)]
            CType::GidT => Ok(LuaValue::Integer(*(ptr as *const libc::gid_t) as i64)),
            #[cfg(unix)]
            CType::ModeT => Ok(LuaValue::Integer(*(ptr as *const libc::mode_t) as i64)),
            #[cfg(unix)]
            CType::NlinkT => Ok(LuaValue::Integer(*(ptr as *const libc::nlink_t) as i64)),
            #[cfg(unix)]
            CType::UidT => Ok(LuaValue::Integer(*(ptr as *const libc::uid_t) as i64)),
            #[cfg(unix)]
            CType::OffT => Ok(LuaValue::Integer(*(ptr as *const libc::off_t) as i64)),
            #[cfg(unix)]
            CType::PidT => Ok(LuaValue::Integer(*(ptr as *const libc::pid_t) as i64)),
            #[cfg(unix)]
            CType::UsecondsT => Ok(LuaValue::Integer(*(ptr as *const libc::useconds_t) as i64)),
            #[cfg(unix)]
            CType::SusecondsT => Ok(LuaValue::Integer(*(ptr as *const libc::suseconds_t) as i64)),
            #[cfg(unix)]
            CType::BlksizeT => Ok(LuaValue::Integer(*(ptr as *const libc::blksize_t) as i64)),
            #[cfg(unix)]
            CType::BlkcntT => Ok(LuaValue::Integer(*(ptr as *const libc::blkcnt_t) as i64)),
            #[cfg(unix)]
            CType::TimeT => Ok(LuaValue::Integer(*(ptr as *const libc::time_t) as i64)),
            
            _ => {
                // For complex types (Ptr, Array, Struct, Union, etc.), return as CData userdata
                let cdata = CData::from_ptr(ctype.clone(), ptr, false);
                lua.create_userdata(cdata).map(|ud| LuaValue::UserData(ud))
            }
        }
    }
}

// Small buffer optimization - avoid heap allocation for small objects
const SMALL_BUFFER_SIZE: usize = 64;

#[derive(Clone)]
pub struct CData {
    pub ctype: CType,
    pub ptr: *mut u8,
    pub owned: bool,
    pub size: usize,
    // Small buffer optimization: store small data inline
    small_buffer: Option<Box<[u8; SMALL_BUFFER_SIZE]>>,
}

impl CData {
    #[inline]
    pub fn new(ctype: CType, size: usize) -> Self {
        // Use small buffer optimization for objects <= 64 bytes
        if size <= SMALL_BUFFER_SIZE && size > 0 {
            let mut buffer = Box::new([0u8; SMALL_BUFFER_SIZE]);
            let ptr = buffer.as_mut_ptr();
            Self {
                ctype,
                ptr,
                owned: true,
                size,
                small_buffer: Some(buffer),
            }
        } else if size > 0 {
            let layout = std::alloc::Layout::from_size_align(size, ctype.alignment())
                .expect("Invalid layout");
            // Use alloc instead of alloc_zeroed for better performance when initialization is not needed
            let ptr = unsafe { std::alloc::alloc(layout) };
            Self {
                ctype,
                ptr,
                owned: true,
                size,
                small_buffer: None,
            }
        } else {
            Self {
                ctype,
                ptr: ptr::null_mut(),
                owned: false,
                size: 0,
                small_buffer: None,
            }
        }
    }

    pub fn new_null_ptr() -> Self {
        Self {
            ctype: CType::Ptr(Box::new(CType::Void)),
            ptr: ptr::null_mut(),
            owned: false,
            size: std::mem::size_of::<*const ()>(),
            small_buffer: None,
        }
    }

    #[inline]
    pub fn from_ptr(ctype: CType, ptr: *mut u8, owned: bool) -> Self {
        let size = ctype.size();
        Self {
            ctype,
            ptr,
            owned,
            size,
            small_buffer: None,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

impl Drop for CData {
    fn drop(&mut self) {
        // If we're using small_buffer, it will be dropped automatically
        // Only deallocate if we're using heap-allocated memory
        if self.owned && !self.ptr.is_null() && self.size > 0 && self.small_buffer.is_none() {
            let layout = std::alloc::Layout::from_size_align(self.size, self.ctype.alignment())
                .expect("Invalid layout");
            unsafe {
                std::alloc::dealloc(self.ptr, layout);
            }
        }
    }
}

impl LuaUserData for CData {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(
            LuaMetaMethod::Index,
            |_lua, this, key: LuaValue| match key {
                LuaValue::String(s) => {
                    let field_name = s.to_str()?;
                    match &this.ctype {
                        CType::Struct(_, fields) | CType::Union(_, fields) => {
                            for field in fields {
                                if field_name == field.name.as_str() {
                                    let field_ptr = unsafe { this.ptr.add(field.offset) };
                                    return read_ctype_value(_lua, field_ptr, &field.ctype);
                                }
                            }
                            Err(LuaError::RuntimeError(format!(
                                "Unknown field: {}",
                                field_name
                            )))
                        }
                        _ => Err(LuaError::RuntimeError("Not a struct or union".to_string())),
                    }
                }
                LuaValue::Integer(i) => {
                    match &this.ctype {
                        CType::Array(elem_type, _) | CType::Ptr(elem_type) => {
                            let elem_size = elem_type.size();
                            let offset = i as usize * elem_size;
                            let elem_ptr = unsafe { this.ptr.add(offset) };
                            read_ctype_value(_lua, elem_ptr, elem_type)
                        }
                        _ => Err(LuaError::RuntimeError(
                            "Not an array or pointer".to_string(),
                        )),
                    }
                }
                _ => Err(LuaError::RuntimeError("Invalid index type".to_string())),
            },
        );

        methods.add_meta_method_mut(
            LuaMetaMethod::NewIndex,
            |_lua, this, (key, value): (LuaValue, LuaValue)| {
                match key {
                    LuaValue::String(s) => {
                        // Field assignment for structs/unions
                        let field_name = s.to_str()?;
                        match &this.ctype {
                            CType::Struct(_, fields) | CType::Union(_, fields) => {
                                for field in fields {
                                    if field_name == field.name.as_str() {
                                        let field_ptr = unsafe { this.ptr.add(field.offset) };
                                        write_value_to_ptr(field_ptr, &field.ctype, value)?;
                                        return Ok(());
                                    }
                                }
                                Err(LuaError::RuntimeError(format!(
                                    "Unknown field: {}",
                                    field_name
                                )))
                            }
                            _ => Err(LuaError::RuntimeError("Not a struct or union".to_string())),
                        }
                    }
                    LuaValue::Integer(i) => {
                        // Array/pointer element assignment
                        match &this.ctype {
                            CType::Array(elem_type, _) | CType::Ptr(elem_type) => {
                                let elem_size = elem_type.size();
                                let offset = i as usize * elem_size;
                                let elem_ptr = unsafe { this.ptr.add(offset) };
                                write_value_to_ptr(elem_ptr, elem_type, value)?;
                                Ok(())
                            }
                            _ => Err(LuaError::RuntimeError(
                                "Not an array or pointer".to_string(),
                            )),
                        }
                    }
                    _ => Err(LuaError::RuntimeError("Invalid index type".to_string())),
                }
            },
        );

        methods.add_meta_method(LuaMetaMethod::Len, |_lua, this, ()| match &this.ctype {
            CType::Array(_, count) => Ok(*count),
            _ => Err(LuaError::RuntimeError("Not an array".to_string())),
        });
    }
}

pub struct CFunction {
    _ptr: *mut libc::c_void,
    pub name: String,
}

impl LuaUserData for CFunction {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Call, |_lua, this, _args: LuaMultiValue| -> LuaResult<LuaValue> {
            Err(LuaError::RuntimeError(format!(
                "C function call not yet fully implemented for '{}'",
                this.name
            )))
        });
    }
}

pub struct CLib {
    handle: Option<DynamicLibrary>,
    _name: String,
}

impl CLib {
    pub fn load(name: &str) -> Result<Self, String> {
        let lib = DynamicLibrary::load(name)?;
        Ok(Self {
            handle: Some(lib),
            _name: name.to_string(),
        })
    }

    pub fn load_default() -> Result<Self, String> {
        let lib = DynamicLibrary::load_default()?;
        Ok(Self {
            handle: Some(lib),
            _name: "C".to_string(),
        })
    }

    pub fn get_symbol(&self, name: &str) -> Option<*mut libc::c_void> {
        self.handle.as_ref()?.get_symbol(name)
    }
}

impl LuaUserData for CLib {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, name: String| {
            if let Some(sym) = this.get_symbol(&name) {
                // Return a callable function wrapper
                let cfunc = CFunction {
                    _ptr: sym,
                    name: name.clone(),
                };
                lua.create_userdata(cfunc)
                    .map(|ud| LuaValue::UserData(ud))
            } else {
                Err(LuaError::RuntimeError(format!(
                    "Symbol not found: {}",
                    name
                )))
            }
        });
    }
}

// Improved macro with better error messages
macro_rules! write_numeric {
    ($ptr:expr, $ty:ty, $value:expr) => {{
        let val = match $value {
            LuaValue::Integer(i) => i as $ty,
            LuaValue::Number(n) => n as $ty,
            _ => return Err(LuaError::RuntimeError(
                format!("Expected number for {} type", stringify!($ty))
            )),
        };
        *($ptr as *mut $ty) = val;
    }};
}

// Improved write function with better type safety and error handling
#[inline]
fn write_value_to_ptr(ptr: *mut u8, ctype: &CType, value: LuaValue) -> LuaResult<()> {
    unsafe {
        match ctype {
            // Basic integer types
            CType::Int => write_numeric!(ptr, i32, value),
            CType::UInt => write_numeric!(ptr, u32, value),
            CType::Long => write_numeric!(ptr, isize, value),
            CType::ULong => write_numeric!(ptr, usize, value),
            CType::LongLong => write_numeric!(ptr, i64, value),
            CType::ULongLong => write_numeric!(ptr, u64, value),
            
            // Character types
            CType::Char => write_numeric!(ptr, i8, value),
            CType::UChar => write_numeric!(ptr, u8, value),
            
            // Short types
            CType::Short => write_numeric!(ptr, i16, value),
            CType::UShort => write_numeric!(ptr, u16, value),
            
            // Fixed-width integer types
            CType::Int8 => write_numeric!(ptr, i8, value),
            CType::Int16 => write_numeric!(ptr, i16, value),
            CType::Int32 => write_numeric!(ptr, i32, value),
            CType::Int64 => write_numeric!(ptr, i64, value),
            CType::UInt8 => write_numeric!(ptr, u8, value),
            CType::UInt16 => write_numeric!(ptr, u16, value),
            CType::UInt32 => write_numeric!(ptr, u32, value),
            CType::UInt64 => write_numeric!(ptr, u64, value),
            
            // Size types
            CType::SizeT => write_numeric!(ptr, usize, value),
            CType::SSizeT => write_numeric!(ptr, isize, value),
            
            // Floating point types
            CType::Float => write_numeric!(ptr, f32, value),
            CType::Double => write_numeric!(ptr, f64, value),
            
            // Boolean type
            CType::Bool => {
                let val = match value {
                    LuaValue::Boolean(b) => b,
                    LuaValue::Integer(i) => i != 0,
                    _ => return Err(LuaError::RuntimeError("Expected boolean or integer".to_string())),
                };
                *(ptr as *mut bool) = val;
            }
            
            // POSIX types (Unix only)
            #[cfg(unix)]
            CType::InoT => write_numeric!(ptr, libc::ino_t, value),
            #[cfg(unix)]
            CType::DevT => write_numeric!(ptr, libc::dev_t, value),
            #[cfg(unix)]
            CType::GidT => write_numeric!(ptr, libc::gid_t, value),
            #[cfg(unix)]
            CType::ModeT => write_numeric!(ptr, libc::mode_t, value),
            #[cfg(unix)]
            CType::NlinkT => write_numeric!(ptr, libc::nlink_t, value),
            #[cfg(unix)]
            CType::UidT => write_numeric!(ptr, libc::uid_t, value),
            #[cfg(unix)]
            CType::OffT => write_numeric!(ptr, libc::off_t, value),
            #[cfg(unix)]
            CType::PidT => write_numeric!(ptr, libc::pid_t, value),
            #[cfg(unix)]
            CType::UsecondsT => write_numeric!(ptr, libc::useconds_t, value),
            #[cfg(unix)]
            CType::SusecondsT => write_numeric!(ptr, libc::suseconds_t, value),
            #[cfg(unix)]
            CType::BlksizeT => write_numeric!(ptr, libc::blksize_t, value),
            #[cfg(unix)]
            CType::BlkcntT => write_numeric!(ptr, libc::blkcnt_t, value),
            #[cfg(unix)]
            CType::TimeT => write_numeric!(ptr, libc::time_t, value),
            
            // Pointer type
            CType::Ptr(_) => {
                match value {
                    LuaValue::Integer(i) => *(ptr as *mut usize) = i as usize,
                    LuaValue::UserData(ud) => {
                        let cdata = ud.borrow::<CData>()?;
                        *(ptr as *mut *mut u8) = cdata.as_ptr();
                    }
                    _ => return Err(LuaError::RuntimeError(
                        "Expected pointer value (integer or cdata)".to_string()
                    )),
                }
            }
            
            _ => return Err(LuaError::RuntimeError(
                format!("Cannot assign value to type: {:?}", ctype)
            )),
        }
    }
    Ok(())
}
