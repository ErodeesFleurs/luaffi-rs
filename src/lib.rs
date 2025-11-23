mod cdata;
pub mod ctype;
mod dylib;
mod ffi_ops;
mod parser;

use mlua::prelude::*;

const LUA_FFI_VERSION: &str = "0.1.1-rust";

/// Create the FFI module with all exported functions
pub fn lua_module(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    exports.set("VERSION", LUA_FFI_VERSION)?;

    // Core FFI functions
    exports.set("cdef", lua.create_function(ffi_cdef)?)?;
    exports.set("load", lua.create_function(ffi_load)?)?;
    exports.set("new", lua.create_function(ffi_new)?)?;
    exports.set("cast", lua.create_function(ffi_cast)?)?;
    exports.set("metatype", lua.create_function(ffi_metatype)?)?;
    exports.set("typeof", lua.create_function(ffi_typeof)?)?;
    
    // Memory operations
    exports.set("addressof", lua.create_function(ffi_addressof)?)?;
    exports.set("gc", lua.create_function(ffi_gc)?)?;
    exports.set("sizeof", lua.create_function(ffi_sizeof)?)?;
    exports.set("offsetof", lua.create_function(ffi_offsetof)?)?;
    
    // Type checking and conversion
    exports.set("istype", lua.create_function(ffi_istype)?)?;
    exports.set("tonumber", lua.create_function(ffi_tonumber)?)?;
    exports.set("string", lua.create_function(ffi_string)?)?;
    
    // Buffer operations
    exports.set("copy", lua.create_function(ffi_copy)?)?;
    exports.set("fill", lua.create_function(ffi_fill)?)?;
    
    // System operations
    exports.set("errno", lua.create_function(ffi_errno)?)?;

    // Constants
    let nullptr = cdata::CData::new_null_ptr();
    exports.set("nullptr", lua.create_userdata(nullptr)?)?;

    // Default C library
    let c_lib = cdata::CLib::load_default().map_err(LuaError::RuntimeError)?;
    exports.set("C", lua.create_userdata(c_lib)?)?;

    Ok(exports)
}

fn init(state: *mut mlua::lua_State) -> libc::c_int {
    unsafe { mlua::Lua::entrypoint1(state, lua_module) }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn luaopen_luaffi(state: *mut mlua::lua_State) -> libc::c_int {
    init(state)
}

/// Parse C definitions and register types
fn ffi_cdef(_lua: &Lua, code: String) -> LuaResult<()> {
    parser::parse_cdef(&code)
        .map_err(|e| LuaError::RuntimeError(format!("Failed to parse C definitions: {}", e)))
}

/// Load a dynamic library by name
fn ffi_load(_lua: &Lua, name: String) -> LuaResult<LuaAnyUserData> {
    let lib = cdata::CLib::load(&name)
        .map_err(|e| LuaError::RuntimeError(format!("Failed to load library '{}': {}", name, e)))?;
    _lua.create_userdata(lib)
}

#[inline]
fn ffi_new(lua: &Lua, (type_name, init): (String, Option<LuaValue>)) -> LuaResult<LuaAnyUserData> {
    ffi_ops::new_cdata(lua, &type_name, init)
}

#[inline]
fn ffi_cast(lua: &Lua, (type_name, value): (String, LuaValue)) -> LuaResult<LuaAnyUserData> {
    ffi_ops::cast_cdata(lua, &type_name, value)
}

fn ffi_metatype(lua: &Lua, (type_name, metatable): (String, LuaTable)) -> LuaResult<LuaValue> {
    ffi_ops::set_metatype(lua, &type_name, metatable)
}

fn ffi_typeof(_lua: &Lua, type_name: String) -> LuaResult<String> {
    Ok(type_name)
}

fn ffi_addressof(lua: &Lua, cdata: LuaAnyUserData) -> LuaResult<LuaAnyUserData> {
    ffi_ops::get_address(lua, cdata)
}

fn ffi_gc(
    lua: &Lua,
    (cdata, finalizer): (LuaAnyUserData, LuaFunction),
) -> LuaResult<LuaAnyUserData> {
    ffi_ops::set_gc(lua, cdata, Some(finalizer))
}

#[inline]
fn ffi_sizeof(_lua: &Lua, type_name: String) -> LuaResult<usize> {
    ffi_ops::sizeof_type(&type_name)
}

fn ffi_offsetof(_lua: &Lua, (type_name, field): (String, String)) -> LuaResult<usize> {
    ffi_ops::offsetof_field(&type_name, &field)
}

fn ffi_istype(_lua: &Lua, (type_name, value): (String, LuaValue)) -> LuaResult<bool> {
    // Check if value is a CData with the specified type
    match value {
        LuaValue::UserData(ud) => {
            if let Ok(cdata) = ud.borrow::<cdata::CData>() {
                // Try to parse the expected type
                match ffi_ops::lookup_type(&type_name) {
                    Ok(expected_type) => Ok(cdata.ctype == expected_type),
                    Err(_) => Ok(false),
                }
            } else {
                Ok(false)
            }
        }
        _ => Ok(false),
    }
}

fn ffi_tonumber(_lua: &Lua, cdata: LuaAnyUserData) -> LuaResult<f64> {
    ffi_ops::cdata_to_number(cdata)
}

fn ffi_string(_lua: &Lua, cdata: LuaAnyUserData) -> LuaResult<String> {
    ffi_ops::cdata_to_string(cdata)
}

fn ffi_copy(
    _lua: &Lua,
    (dst, src, len): (LuaAnyUserData, LuaValue, Option<usize>),
) -> LuaResult<usize> {
    ffi_ops::copy_memory(dst, src, len)
}

fn ffi_fill(_lua: &Lua, (cdata, len, value): (LuaAnyUserData, usize, Option<u8>)) -> LuaResult<()> {
    ffi_ops::fill_memory(cdata, len, value.unwrap_or(0))
}

fn ffi_errno(_lua: &Lua, _new_errno: Option<i32>) -> LuaResult<i32> {
    #[cfg(unix)]
    {
        unsafe {
            #[cfg(target_os = "linux")]
            {
                let errno_ptr = libc::__errno_location();
                let old_errno = *errno_ptr;
                if let Some(new) = _new_errno {
                    *errno_ptr = new;
                }
                Ok(old_errno)
            }
            #[cfg(not(target_os = "linux"))]
            {
                // For BSD/macOS: errno is accessed differently
                let old_errno = *libc::__error();
                if let Some(new) = _new_errno {
                    *libc::__error() = new;
                }
                Ok(old_errno)
            }
        }
    }
    #[cfg(not(unix))]
    {
        // Windows and other platforms
        Err(LuaError::RuntimeError(
            "errno not supported on this platform".to_string(),
        ))
    }
}
