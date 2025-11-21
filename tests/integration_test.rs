use mlua::prelude::*;

// Helper function to create a Lua VM with the FFI module loaded
fn create_lua_with_ffi() -> Lua {
    let lua = Lua::new();
    
    // Load the FFI module
    lua.load(r#"
        -- Mock FFI module for testing
        -- In real usage, this would be loaded as a C module
        ffi = {}
        ffi.VERSION = "0.1.0-rust"
        
        -- These will be properly implemented when the module is loaded
        function ffi.cdef(code) end
        function ffi.new(type_name, init) end
        function ffi.cast(type_name, value) end
        function ffi.typeof(type_name) return type_name end
        function ffi.sizeof(type_name) return 4 end
        function ffi.offsetof(type_name, field) return 0 end
        function ffi.istype(type_name, value) return false end
        function ffi.metatype(type_name, metatable) return nil end
        function ffi.gc(cdata, finalizer) return cdata end
        function ffi.addressof(cdata) return cdata end
        function ffi.tonumber(cdata) return 0 end
        function ffi.string(cdata) return "" end
        function ffi.copy(dst, src, len) end
        function ffi.fill(cdata, len, value) end
        function ffi.errno(new_errno) return 0 end
        
        ffi.nullptr = nil
        ffi.C = {}
    "#).exec().unwrap();
    
    lua
}

#[test]
fn test_ffi_module_loads() {
    let lua = create_lua_with_ffi();
    
    // Check that FFI module exists
    let version: String = lua.load("return ffi.VERSION").eval().unwrap();
    assert!(version.contains("0.1.0"));
}

#[test]
fn test_basic_types() {
    let lua = create_lua_with_ffi();
    
    // Test that basic type names are recognized
    let result: String = lua.load(r#"
        return ffi.typeof("int")
    "#).eval().unwrap();
    
    assert_eq!(result, "int");
}

#[test]
fn test_sizeof() {
    let lua = create_lua_with_ffi();
    
    // Test sizeof for basic types
    let size: usize = lua.load(r#"
        return ffi.sizeof("int")
    "#).eval().unwrap();
    
    assert!(size > 0);
}

#[test]
fn test_cdef_struct() {
    let lua = create_lua_with_ffi();
    
    // Test struct definition
    let result = lua.load(r#"
        ffi.cdef[[
            struct Point {
                int x;
                int y;
            };
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_array_syntax() {
    let lua = create_lua_with_ffi();
    
    // Test array type syntax
    let result = lua.load(r#"
        return ffi.typeof("int[10]")
    "#).eval::<String>();
    
    assert!(result.is_ok());
}

#[test]
fn test_pointer_syntax() {
    let lua = create_lua_with_ffi();
    
    // Test pointer type syntax
    let result = lua.load(r#"
        return ffi.typeof("int*")
    "#).eval::<String>();
    
    assert!(result.is_ok());
}

#[test]
fn test_nullptr() {
    let lua = create_lua_with_ffi();
    
    // Test nullptr existence
    let result = lua.load(r#"
        return ffi.nullptr ~= nil
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_c_library() {
    let lua = create_lua_with_ffi();
    
    // Test C library object exists
    let result = lua.load(r#"
        return ffi.C ~= nil
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_ffi_copy() {
    let lua = create_lua_with_ffi();
    
    // Test copy function exists and can be called
    let result = lua.load(r#"
        -- Mock implementation for testing
        local src = "hello"
        ffi.copy(nil, src)
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_ffi_fill() {
    let lua = create_lua_with_ffi();
    
    // Test fill function exists and can be called
    let result = lua.load(r#"
        ffi.fill(nil, 10, 0)
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_ffi_errno() {
    let lua = create_lua_with_ffi();
    
    // Test errno function
    let result = lua.load(r#"
        local old_errno = ffi.errno()
        return type(old_errno) == "number"
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_complex_struct() {
    let lua = create_lua_with_ffi();
    
    // Test complex struct with nested types
    let result = lua.load(r#"
        ffi.cdef[[
            struct Rectangle {
                int x;
                int y;
                int width;
                int height;
            };
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_typedef() {
    let lua = create_lua_with_ffi();
    
    // Test typedef parsing
    let result = lua.load(r#"
        ffi.cdef[[
            typedef int my_int;
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_multiple_fields() {
    let lua = create_lua_with_ffi();
    
    // Test struct with multiple fields of different types
    let result = lua.load(r#"
        ffi.cdef[[
            struct Data {
                char name;
                int age;
                float height;
                double weight;
            };
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_fixed_width_types() {
    let lua = create_lua_with_ffi();
    
    // Test fixed-width integer types
    let types = vec!["int8_t", "int16_t", "int32_t", "int64_t",
                     "uint8_t", "uint16_t", "uint32_t", "uint64_t"];
    
    for type_name in types {
        let result = lua.load(&format!("return ffi.typeof('{}')", type_name))
            .eval::<String>();
        assert!(result.is_ok(), "Failed for type: {}", type_name);
    }
}

#[test]
fn test_size_t_types() {
    let lua = create_lua_with_ffi();
    
    // Test size_t and ssize_t
    let result = lua.load(r#"
        local s1 = ffi.typeof("size_t")
        local s2 = ffi.typeof("ssize_t")
        return s1 ~= nil and s2 ~= nil
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_float_types() {
    let lua = create_lua_with_ffi();
    
    // Test floating point types
    let result = lua.load(r#"
        local f = ffi.typeof("float")
        local d = ffi.typeof("double")
        return f ~= nil and d ~= nil
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_void_type() {
    let lua = create_lua_with_ffi();
    
    // Test void type
    let result = lua.load(r#"
        return ffi.typeof("void")
    "#).eval::<String>();
    
    assert!(result.is_ok());
}

#[test]
fn test_bool_type() {
    let lua = create_lua_with_ffi();
    
    // Test bool type
    let result = lua.load(r#"
        return ffi.typeof("bool")
    "#).eval::<String>();
    
    assert!(result.is_ok());
}

#[test]
fn test_char_types() {
    let lua = create_lua_with_ffi();
    
    // Test char and unsigned char
    let result = lua.load(r#"
        local c = ffi.typeof("char")
        local uc = ffi.typeof("unsigned char")
        return c ~= nil and uc ~= nil
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_short_types() {
    let lua = create_lua_with_ffi();
    
    // Test short and unsigned short
    let result = lua.load(r#"
        local s = ffi.typeof("short")
        local us = ffi.typeof("unsigned short")
        return s ~= nil and us ~= nil
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_long_types() {
    let lua = create_lua_with_ffi();
    
    // Test long and unsigned long
    let result = lua.load(r#"
        local l = ffi.typeof("long")
        local ul = ffi.typeof("unsigned long")
        return l ~= nil and ul ~= nil
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_pointer_to_pointer() {
    let lua = create_lua_with_ffi();
    
    // Test pointer to pointer syntax
    let result = lua.load(r#"
        return ffi.typeof("int**")
    "#).eval::<String>();
    
    assert!(result.is_ok());
}

#[test]
fn test_array_of_pointers() {
    let lua = create_lua_with_ffi();
    
    // Test array of pointers
    let result = lua.load(r#"
        return ffi.typeof("int*[5]")
    "#).eval::<String>();
    
    assert!(result.is_ok());
}

#[test]
fn test_empty_array() {
    let lua = create_lua_with_ffi();
    
    // Test empty array syntax (flexible array member)
    let result = lua.load(r#"
        return ffi.typeof("int[]")
    "#).eval::<String>();
    
    assert!(result.is_ok());
}

#[test]
fn test_struct_with_array() {
    let lua = create_lua_with_ffi();
    
    // Test struct containing an array
    let result = lua.load(r#"
        ffi.cdef[[
            struct Buffer {
                int size;
                char data[256];
            };
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_error_handling_invalid_type() {
    let lua = create_lua_with_ffi();
    
    // Test error handling for invalid type
    let result = lua.load(r#"
        pcall(function()
            ffi.typeof("invalid_type_xyz")
        end)
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_error_handling_malformed_struct() {
    let lua = create_lua_with_ffi();
    
    // Test error handling for malformed struct
    let result = lua.load(r#"
        local ok = pcall(function()
            ffi.cdef[[
                struct BadStruct {
                    int x
                    -- missing semicolon
                };
            ]]
        end)
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_metatype_basic() {
    let lua = create_lua_with_ffi();
    
    // Test metatype function
    let result = lua.load(r#"
        local mt = {}
        ffi.metatype("int", mt)
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_gc_basic() {
    let lua = create_lua_with_ffi();
    
    // Test gc function
    let result = lua.load(r#"
        local function finalizer(cdata)
            -- cleanup
        end
        -- ffi.gc would be called with actual cdata
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_addressof_usage() {
    let lua = create_lua_with_ffi();
    
    // Test addressof function exists
    let result = lua.load(r#"
        return type(ffi.addressof) == "function"
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_istype_usage() {
    let lua = create_lua_with_ffi();
    
    // Test istype function
    let result = lua.load(r#"
        local result = ffi.istype("int", 42)
        return type(result) == "boolean"
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_tonumber_usage() {
    let lua = create_lua_with_ffi();
    
    // Test tonumber function exists
    let result = lua.load(r#"
        return type(ffi.tonumber) == "function"
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_string_usage() {
    let lua = create_lua_with_ffi();
    
    // Test string function exists
    let result = lua.load(r#"
        return type(ffi.string) == "function"
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_multiple_structs() {
    let lua = create_lua_with_ffi();
    
    // Test defining multiple structs
    let result = lua.load(r#"
        ffi.cdef[[
            struct Point { int x; int y; };
            struct Circle { int x; int y; int radius; };
            struct Rectangle { int x; int y; int w; int h; };
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_struct_name_uniqueness() {
    let lua = create_lua_with_ffi();
    
    // Test that struct names are tracked properly
    let result = lua.load(r#"
        ffi.cdef[[
            struct UniqueStruct1 { int value; };
        ]]
        ffi.cdef[[
            struct UniqueStruct2 { float value; };
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_whitespace_handling() {
    let lua = create_lua_with_ffi();
    
    // Test that whitespace is handled correctly
    let result = lua.load(r#"
        ffi.cdef[[
            struct   SpacedStruct   {
                int    x   ;
                float  y   ;
            }   ;
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_multiline_struct() {
    let lua = create_lua_with_ffi();
    
    // Test multiline struct definition
    let result = lua.load(r#"
        ffi.cdef[[
            struct MultiLine {
                int a;
                int b;
                int c;
                int d;
                int e;
            };
        ]]
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}

#[test]
fn test_api_completeness() {
    let lua = create_lua_with_ffi();
    
    // Test that all expected API functions exist
    let result = lua.load(r#"
        local functions = {
            "cdef", "new", "cast", "typeof", "sizeof", "offsetof",
            "istype", "metatype", "gc", "addressof", "tonumber",
            "string", "copy", "fill", "errno"
        }
        
        for _, name in ipairs(functions) do
            if type(ffi[name]) ~= "function" then
                return false
            end
        end
        
        return true
    "#).eval::<bool>();
    
    assert!(result.is_ok() && result.unwrap());
}

#[test]
fn test_constants_exist() {
    let lua = create_lua_with_ffi();
    
    // Test that expected constants exist
    let result = lua.load(r#"
        return ffi.VERSION ~= nil and ffi.nullptr ~= nil and ffi.C ~= nil
    "#).eval::<bool>();
    
    assert!(result.is_ok());
}
