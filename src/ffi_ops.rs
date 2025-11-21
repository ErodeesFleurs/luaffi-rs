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
pub fn new_cdata(lua: &Lua, type_name: &str, _init: Option<LuaValue>) -> LuaResult<LuaAnyUserData> {
    let ctype = lookup_type(type_name)?;
    let size = ctype.size();

    let cdata = CData::new(ctype, size);

    lua.create_userdata(cdata)
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
        CType::Ptr(inner) | CType::Array(inner, _) => match **inner {
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
    // Check basic types first (fastest path)
    if let Some(ctype) = lookup_basic_type(type_name) {
        return Ok(ctype);
    }

    // Check for pointer type
    if type_name.ends_with('*') {
        let base_type = type_name.trim_end_matches('*').trim();
        let inner = lookup_type(base_type)?;
        return Ok(CType::Ptr(Box::new(inner)));
    }

    // Check for array type
    if let Some(open_bracket) = type_name.find('[') {
        let base_type = type_name[..open_bracket].trim();
        let inner = lookup_type(base_type)?;

        let close_bracket = type_name.find(']').ok_or_else(|| {
            LuaError::RuntimeError(format!("Invalid array type (missing ']'): {}", type_name))
        })?;

        let size_str = type_name[open_bracket + 1..close_bracket].trim();
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
    lookup_registered_type(type_name)
        .ok_or_else(|| LuaError::RuntimeError(format!("Unknown type: {}", type_name)))
}
