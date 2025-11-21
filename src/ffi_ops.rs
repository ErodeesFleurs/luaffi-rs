use std::ffi::CStr;
use std::sync::{RwLock, OnceLock};
use std::collections::HashMap;

use mlua::prelude::*;
use phf::phf_map;

use crate::cdata::CData;
use crate::ctype::CType;

// Static perfect hash map for basic type lookups (zero overhead)
static BASIC_TYPES: phf::Map<&'static str, CType> = phf_map! {
    "int" => CType::Int,
    "unsigned int" => CType::UInt,
    "char" => CType::Char,
    "unsigned char" => CType::UChar,
    "short" => CType::Short,
    "unsigned short" => CType::UShort,
    "long" => CType::Long,
    "unsigned long" => CType::ULong,
    "float" => CType::Float,
    "double" => CType::Double,
    "void" => CType::Void,
    "bool" => CType::Bool,
    "int8_t" => CType::Int8,
    "int16_t" => CType::Int16,
    "int32_t" => CType::Int32,
    "int64_t" => CType::Int64,
    "uint8_t" => CType::UInt8,
    "uint16_t" => CType::UInt16,
    "uint32_t" => CType::UInt32,
    "uint64_t" => CType::UInt64,
    "size_t" => CType::SizeT,
    "ssize_t" => CType::SSizeT,
};

// Global type registry for storing parsed types (using RwLock for better concurrent read performance)
static TYPE_REGISTRY: OnceLock<RwLock<HashMap<String, CType>>> = OnceLock::new();
pub fn register_type(name: String, ctype: CType) {
    TYPE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new())).write().unwrap().insert(name, ctype);
}

#[inline]
fn lookup_registered_type(name: &str) -> Option<CType> {
    TYPE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new())).read().unwrap().get(name).cloned()
}
pub fn new_cdata(lua: &Lua, type_name: &str, init: Option<LuaValue>) -> LuaResult<LuaAnyUserData> {
    let ctype = lookup_type(type_name)?;
    
    // Handle VLA: extract size from init parameter
    let (actual_ctype, size, actual_init) = match &ctype {
        CType::VLA(elem_type) => {
            // For VLA, init must be a number (integer or float) specifying the array size
            let count = match init {
                Some(LuaValue::Integer(i)) if i >= 0 => i as usize,
                Some(LuaValue::Number(n)) if n >= 0.0 && n.is_finite() => n as usize,
                Some(LuaValue::Integer(_)) | Some(LuaValue::Number(_)) => {
                    return Err(LuaError::RuntimeError(
                        "VLA size must be non-negative".to_string()
                    ));
                }
                Some(_) => {
                    return Err(LuaError::RuntimeError(
                        "VLA requires a numeric size as initialization parameter".to_string()
                    ));
                }
                _ => {
                    return Err(LuaError::RuntimeError(
                        "VLA requires a size parameter: ffi.new('type[?]', size)".to_string()
                    ));
                }
            };
            
            let elem_size = elem_type.size();
            let total_size = elem_size * count;
            // Convert VLA to Array with actual size
            let array_type = CType::Array(elem_type.clone(), count);
            (array_type, total_size, None)
        }
        _ => {
            let size = ctype.size();
            (ctype.clone(), size, init)
        }
    };

    let mut cdata = CData::new(actual_ctype, size);

    // Initialize the memory if init value is provided
    if let Some(init_value) = actual_init {
        initialize_cdata(&mut cdata, init_value)?;
    }

    lua.create_userdata(cdata)
}

// Macro for writing numeric values
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

// Write a Lua value to memory at the given pointer
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
            CType::Ptr(inner_type) => {
                match value {
                    LuaValue::Integer(i) => *(ptr as *mut usize) = i as usize,
                    LuaValue::UserData(ud) => {
                        let cdata = ud.borrow::<CData>()?;
                        *(ptr as *mut *mut u8) = cdata.as_ptr();
                    }
                    LuaValue::String(s) if matches!(**inner_type, CType::Char | CType::UChar) => {
                        // String literal assignment to char* pointer
                        // Note: This creates a pointer to the string's data, which may be temporary
                        // In a real implementation, you'd need to manage string lifetime
                        let bytes = s.as_bytes();
                        *(ptr as *mut *const u8) = bytes.as_ptr();
                    }
                    LuaValue::Nil => {
                        // NULL pointer assignment
                        *(ptr as *mut usize) = 0;
                    }
                    _ => return Err(LuaError::RuntimeError(
                        "Expected pointer value (integer, cdata, string, or nil)".to_string()
                    )),
                }
            }
            
            // VLA type - should not reach here as VLA is converted to Array in new_cdata
            CType::VLA(_) => {
                return Err(LuaError::RuntimeError(
                    "VLA must be instantiated with a size before use".to_string()
                ));
            }
            
            // Array type - initialize from table
            CType::Array(elem_type, count) => {
                match value {
                    LuaValue::Table(table) => {
                        let elem_size = elem_type.size();
                        for i in 0..*count {
                            // Lua tables are 1-indexed
                            if let Ok(elem_value) = table.get::<LuaValue>(i + 1) {
                                let elem_ptr = ptr.add(i * elem_size);
                                write_value_to_ptr(elem_ptr, elem_type, elem_value)?;
                            }
                        }
                    }
                    LuaValue::String(s) => {
                        // String initialization for char arrays
                        if matches!(**elem_type, CType::Char | CType::UChar) {
                            let bytes = s.as_bytes();
                            let copy_len = (*count).min(bytes.len());
                            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, copy_len);
                            // Null-terminate if there's space
                            if copy_len < *count {
                                *ptr.add(copy_len) = 0;
                            }
                        } else {
                            return Err(LuaError::RuntimeError(
                                "String initialization only supported for char arrays".to_string()
                            ));
                        }
                    }
                    _ => {
                        return Err(LuaError::RuntimeError(
                            "Array initialization requires a table or string (for char arrays)".to_string()
                        ));
                    }
                }
            }
            
            // Struct type - initialize from table
            CType::Struct(_, fields) => {
                if let LuaValue::Table(table) = value {
                    for field in fields {
                        if let Ok(field_value) = table.get::<LuaValue>(field.name.as_str()) {
                            let field_ptr = ptr.add(field.offset);
                            write_value_to_ptr(field_ptr, &field.ctype, field_value)?;
                        }
                    }
                } else {
                    return Err(LuaError::RuntimeError(
                        "Struct initialization requires a table".to_string()
                    ));
                }
            }
            
            // Union type - initialize from table (typically first field or named field)
            CType::Union(_, fields) => {
                if let LuaValue::Table(table) = value {
                    // Try to find a matching field name in the table
                    for field in fields {
                        if let Ok(field_value) = table.get::<LuaValue>(field.name.as_str()) {
                            let field_ptr = ptr.add(field.offset);
                            write_value_to_ptr(field_ptr, &field.ctype, field_value)?;
                            // For unions, we only initialize one field
                            break;
                        }
                    }
                } else {
                    return Err(LuaError::RuntimeError(
                        "Union initialization requires a table".to_string()
                    ));
                }
            }
            
            // Typedef - unwrap and write to the underlying type
            CType::Typedef(_, inner_type) => {
                write_value_to_ptr(ptr, inner_type, value)?;
            }
            
            // Void type - cannot write
            CType::Void => {
                return Err(LuaError::RuntimeError(
                    "Cannot assign value to void type".to_string()
                ));
            }
            
            // Function type - assign function pointer
            CType::Function(_, _) => {
                match value {
                    LuaValue::Integer(i) => *(ptr as *mut usize) = i as usize,
                    LuaValue::UserData(ud) => {
                        let cdata = ud.borrow::<CData>()?;
                        *(ptr as *mut *mut u8) = cdata.as_ptr();
                    }
                    _ => return Err(LuaError::RuntimeError(
                        "Function pointer requires integer or cdata".to_string()
                    )),
                }
            }
        }
    }
    Ok(())
}

// Helper function to initialize CData with a value
fn initialize_cdata(cdata: &mut CData, value: LuaValue) -> LuaResult<()> {
    if cdata.ptr.is_null() || cdata.size == 0 {
        return Ok(());
    }

    match &cdata.ctype {
        CType::Struct(_, fields) | CType::Union(_, fields) => {
            // Initialize struct/union fields from a table
            if let LuaValue::Table(table) = value {
                for field in fields {
                    if let Ok(field_value) = table.get::<LuaValue>(field.name.as_str()) {
                        let field_ptr = unsafe { cdata.ptr.add(field.offset) };
                        write_value_to_ptr(field_ptr, &field.ctype, field_value)?;
                    }
                }
            } else {
                return Err(LuaError::RuntimeError(
                    "Struct/union initialization requires a table".to_string()
                ));
            }
        }
        CType::Array(elem_type, count) => {
            // Initialize array elements from a table
            if let LuaValue::Table(table) = value {
                let elem_size = elem_type.size();
                for i in 0..*count {
                    // Lua tables are 1-indexed
                    if let Ok(elem_value) = table.get::<LuaValue>(i + 1) {
                        let elem_ptr = unsafe { cdata.ptr.add(i * elem_size) };
                        write_value_to_ptr(elem_ptr, elem_type, elem_value)?;
                    }
                }
            } else {
                return Err(LuaError::RuntimeError(
                    "Array initialization requires a table".to_string()
                ));
            }
        }
        _ => {
            // Initialize scalar types directly
            write_value_to_ptr(cdata.ptr, &cdata.ctype, value)?;
        }
    }
    Ok(())
}

pub fn cast_cdata(lua: &Lua, type_name: &str, value: LuaValue) -> LuaResult<LuaAnyUserData> {
    let ctype = lookup_type(type_name)?;

    let ptr = match value {
        LuaValue::Integer(i) => i as *mut u8,
        LuaValue::UserData(ud) => {
            let cdata = ud.borrow::<CData>()?;
            cdata.as_ptr()
        }
        _ => return Err(LuaError::RuntimeError("Cannot cast this value".to_string())),
    };

    let cdata = CData::from_ptr(ctype, ptr, false);
    lua.create_userdata(cdata)
}

pub fn set_metatype(lua: &Lua, type_name: &str, metatable: LuaTable) -> LuaResult<LuaValue> {
    // Store the metatable in the Lua registry with a key based on type name
    let registry_key = format!("ffi_metatype_{}", type_name);
    lua.set_named_registry_value(&registry_key, metatable.clone())?;
    
    // Return the metatable
    Ok(LuaValue::Table(metatable))
}

pub fn get_address(lua: &Lua, cdata: LuaAnyUserData) -> LuaResult<LuaAnyUserData> {
    let cd = cdata.borrow::<CData>()?;
    let ptr_type = CType::Ptr(Box::new(cd.ctype.clone()));
    let addr_cdata = CData::from_ptr(ptr_type, cd.as_ptr(), false);
    lua.create_userdata(addr_cdata)
}

pub fn set_gc(
    lua: &Lua,
    cdata: LuaAnyUserData,
    finalizer: Option<LuaFunction>,
) -> LuaResult<LuaAnyUserData> {
    // Store the finalizer in a registry table associated with the userdata
    // This is a workaround since mlua doesn't allow direct metatable modification
    if let Some(fin) = finalizer {
        // Create a unique key for this userdata in the registry
        let registry_key = format!("ffi_gc_{:p}", cdata.to_pointer());
        lua.set_named_registry_value(&registry_key, fin)?;
        
        // Note: In a complete implementation, we would need to modify the CData
        // struct to store a flag indicating it has a finalizer, and call it in Drop
    }
    Ok(cdata)
}

pub fn sizeof_type(type_name: &str) -> LuaResult<usize> {
    let ctype = lookup_type(type_name)?;
    Ok(ctype.size())
}

pub fn offsetof_field(type_name: &str, field: &str) -> LuaResult<usize> {
    let ctype = lookup_type(type_name)?;

    match ctype {
        CType::Struct(_, fields) | CType::Union(_, fields) => {
            for f in fields {
                if f.name == field {
                    return Ok(f.offset);
                }
            }
            Err(LuaError::RuntimeError(format!(
                "Field not found: {}",
                field
            )))
        }
        _ => Err(LuaError::RuntimeError("Not a struct or union".to_string())),
    }
}

pub fn cdata_to_number(cdata: LuaAnyUserData) -> LuaResult<f64> {
    let cd = cdata.borrow::<CData>()?;

    if cd.is_null() {
        return Ok(0.0);
    }

    // Validate buffer has enough data for the type
    let type_size = cd.ctype.size();
    if cd.size < type_size {
        return Err(LuaError::RuntimeError(format!(
            "Buffer too small: {} bytes available, {} needed",
            cd.size, type_size
        )));
    }

    unsafe {
        match cd.ctype {
            CType::Int => Ok(*(cd.ptr as *const i32) as f64),
            CType::UInt => Ok(*(cd.ptr as *const u32) as f64),
            CType::Long => Ok(*(cd.ptr as *const isize) as f64),
            CType::Float => Ok(*(cd.ptr as *const f32) as f64),
            CType::Double => Ok(*(cd.ptr as *const f64)),
            CType::Ptr(_) => Ok(cd.ptr as usize as f64),
            _ => Err(LuaError::RuntimeError(
                "Cannot convert to number".to_string(),
            )),
        }
    }
}

pub fn cdata_to_string(cdata: LuaAnyUserData) -> LuaResult<String> {
    let cd = cdata.borrow::<CData>()?;

    if cd.is_null() {
        return Err(LuaError::RuntimeError("NULL pointer".to_string()));
    }

    match &cd.ctype {
        CType::Ptr(inner) | CType::Array(inner, _) | CType::VLA(inner) => match **inner {
            CType::Char | CType::UChar => unsafe {
                let c_str = CStr::from_ptr(cd.ptr as *const i8);
                Ok(c_str.to_string_lossy().to_string())
            },
            _ => Err(LuaError::RuntimeError("Not a string pointer".to_string())),
        },
        _ => Err(LuaError::RuntimeError("Not a string".to_string())),
    }
}

pub fn copy_memory(dst: LuaAnyUserData, src: LuaValue, len: Option<usize>) -> LuaResult<usize> {
    let dst_cd = dst.borrow::<CData>()?;

    match src {
        LuaValue::String(s) => {
            let bytes = s.as_bytes();
            let copy_len = len.unwrap_or(bytes.len());

            // Validate destination buffer size
            if copy_len > dst_cd.size {
                return Err(LuaError::RuntimeError(format!(
                    "Buffer overflow: trying to copy {} bytes to buffer of size {}",
                    copy_len, dst_cd.size
                )));
            }

            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst_cd.ptr, copy_len);
                // Only null-terminate if we have space and it wasn't explicitly specified
                if len.is_none() && copy_len < dst_cd.size {
                    *dst_cd.ptr.add(copy_len) = 0;
                }
            }
            Ok(copy_len)
        }
        LuaValue::UserData(src_ud) => {
            let src_cd = src_ud.borrow::<CData>()?;
            let copy_len = len.ok_or_else(|| {
                LuaError::RuntimeError("Length required for cdata copy".to_string())
            })?;
            unsafe {
                std::ptr::copy_nonoverlapping(src_cd.ptr, dst_cd.ptr, copy_len);
            }
            Ok(copy_len)
        }
        _ => Err(LuaError::RuntimeError(
            "Invalid source for copy".to_string(),
        )),
    }
}

pub fn fill_memory(cdata: LuaAnyUserData, len: usize, value: u8) -> LuaResult<()> {
    let cd = cdata.borrow::<CData>()?;
    unsafe {
        std::ptr::write_bytes(cd.ptr, value, len);
    }
    Ok(())
}

#[inline]
fn lookup_basic_type(type_name: &str) -> Option<CType> {
    BASIC_TYPES.get(type_name).cloned()
}

pub fn lookup_type(type_name: &str) -> LuaResult<CType> {
    // Strip type qualifiers (const, volatile, restrict, etc.)
    let stripped_name = type_name
        .trim()
        .trim_start_matches("const")
        .trim()
        .trim_start_matches("volatile")
        .trim()
        .trim_start_matches("restrict")
        .trim();
    
    // Check basic types first (fastest path)
    if let Some(ctype) = lookup_basic_type(stripped_name) {
        return Ok(ctype);
    }

    // Check for pointer type
    if stripped_name.ends_with('*') {
        let base_type = stripped_name.trim_end_matches('*').trim();
        let inner = lookup_type(base_type)?;
        return Ok(CType::Ptr(Box::new(inner)));
    }

    // Check for array type
    if let Some(open_bracket) = stripped_name.find('[') {
        let base_type = stripped_name[..open_bracket].trim();
        let inner = lookup_type(base_type)?;

        let close_bracket = stripped_name.find(']').ok_or_else(|| {
            LuaError::RuntimeError(format!("Invalid array type (missing ']'): {}", type_name))
        })?;

        let size_str = stripped_name[open_bracket + 1..close_bracket].trim();
        
        // Check for VLA syntax [?]
        if size_str == "?" {
            return Ok(CType::VLA(Box::new(inner)));
        }
        
        let size = if size_str.is_empty() {
            0 // Flexible array
        } else {
            size_str.parse::<usize>().map_err(|_| {
                LuaError::RuntimeError(format!("Invalid array size: '{}'", size_str))
            })?
        };

        return Ok(CType::Array(Box::new(inner), size));
    }

    // Look up in the type registry for structs/typedefs
    lookup_registered_type(stripped_name)
        .ok_or_else(|| LuaError::RuntimeError(format!("Unknown type: {}", type_name)))
}
