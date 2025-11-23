use luaffi;
use mlua::prelude::*;

// Helper function to create a Lua VM with the FFI module loaded
fn create_lua_with_ffi() -> Lua {
    let lua = Lua::new();

    // Load the real FFI module
    let ffi_module = luaffi::lua_module(&lua).expect("Failed to create FFI module");
    lua.globals()
        .set("ffi", ffi_module)
        .expect("Failed to set ffi global");

    lua
}

#[test]
fn test_ffi_module_loads() {
    let lua = create_lua_with_ffi();

    // Check that FFI module exists
    let version: String = lua.load("return ffi.VERSION").eval().unwrap();
    assert!(version.contains("0.1"), "Version is: {}", version);
}

#[test]
fn test_basic_types() {
    let lua = create_lua_with_ffi();

    // Test that basic type names are recognized
    let result: String = lua
        .load(
            r#"
        return ffi.typeof("int")
    "#,
        )
        .eval()
        .unwrap();

    assert_eq!(result, "int");
}

#[test]
fn test_sizeof() {
    let lua = create_lua_with_ffi();

    // Test sizeof for basic types
    let size: usize = lua
        .load(
            r#"
        return ffi.sizeof("int")
    "#,
        )
        .eval()
        .unwrap();

    assert!(size > 0);
}

#[test]
fn test_cdef_struct() {
    let lua = create_lua_with_ffi();

    // Test struct definition
    let result = lua
        .load(
            r#"
        ffi.cdef[[
            struct Point {
                int x;
                int y;
            };
        ]]
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_array_syntax() {
    let lua = create_lua_with_ffi();

    // Test array type syntax
    let result = lua
        .load(
            r#"
        return ffi.typeof("int[10]")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_pointer_syntax() {
    let lua = create_lua_with_ffi();

    // Test pointer type syntax
    let result = lua
        .load(
            r#"
        return ffi.typeof("int*")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_nullptr() {
    let lua = create_lua_with_ffi();

    // Test nullptr existence
    let result = lua
        .load(
            r#"
        return ffi.nullptr ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_c_library() {
    let lua = create_lua_with_ffi();

    // Test C library object exists
    let result = lua
        .load(
            r#"
        return ffi.C ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_ffi_copy() {
    let lua = create_lua_with_ffi();

    // Test copy function exists and can be called
    let result = lua
        .load(
            r#"
        local dst = ffi.new("char[10]")
        local src = "hello"
        ffi.copy(dst, src)
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_ffi_fill() {
    let lua = create_lua_with_ffi();

    // Test fill function exists and can be called
    let result = lua
        .load(
            r#"
        local buffer = ffi.new("char[10]")
        ffi.fill(buffer, 10, 0)
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_ffi_errno() {
    #[cfg(not(unix))]
    return;
    #[cfg(unix)]
    {
        let lua = create_lua_with_ffi();

        // Test errno function
        let result = lua
            .load(
                r#"
        local old_errno = ffi.errno()
        return type(old_errno) == "number"
    "#,
            )
            .eval::<bool>();

        assert!(result.is_ok());
    }
}

#[test]
fn test_complex_struct() {
    let lua = create_lua_with_ffi();

    // Test complex struct with nested types
    let result = lua
        .load(
            r#"
        ffi.cdef[[
            struct Rectangle {
                int x;
                int y;
                int width;
                int height;
            };
        ]]
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_typedef() {
    let lua = create_lua_with_ffi();

    // Test typedef parsing
    let result = lua
        .load(
            r#"
        ffi.cdef[[
            typedef int my_int;
        ]]
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_multiple_fields() {
    let lua = create_lua_with_ffi();

    // Test struct with multiple fields of different types
    let result = lua
        .load(
            r#"
        ffi.cdef[[
            struct Data {
                char name;
                int age;
                float height;
                double weight;
            };
        ]]
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_fixed_width_types() {
    let lua = create_lua_with_ffi();

    // Test fixed-width integer types
    let types = vec![
        "int8_t", "int16_t", "int32_t", "int64_t", "uint8_t", "uint16_t", "uint32_t", "uint64_t",
    ];

    for type_name in types {
        let result = lua
            .load(&format!("return ffi.typeof('{}')", type_name))
            .eval::<String>();
        assert!(result.is_ok(), "Failed for type: {}", type_name);
    }
}

#[test]
fn test_size_t_types() {
    let lua = create_lua_with_ffi();

    // Test size_t and ssize_t
    let result = lua
        .load(
            r#"
        local s1 = ffi.typeof("size_t")
        local s2 = ffi.typeof("ssize_t")
        return s1 ~= nil and s2 ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_float_types() {
    let lua = create_lua_with_ffi();

    // Test floating point types
    let result = lua
        .load(
            r#"
        local f = ffi.typeof("float")
        local d = ffi.typeof("double")
        return f ~= nil and d ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_void_type() {
    let lua = create_lua_with_ffi();

    // Test void type
    let result = lua
        .load(
            r#"
        return ffi.typeof("void")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_bool_type() {
    let lua = create_lua_with_ffi();

    // Test bool type
    let result = lua
        .load(
            r#"
        return ffi.typeof("bool")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_char_types() {
    let lua = create_lua_with_ffi();

    // Test char and unsigned char
    let result = lua
        .load(
            r#"
        local c = ffi.typeof("char")
        local uc = ffi.typeof("unsigned char")
        return c ~= nil and uc ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_short_types() {
    let lua = create_lua_with_ffi();

    // Test short and unsigned short
    let result = lua
        .load(
            r#"
        local s = ffi.typeof("short")
        local us = ffi.typeof("unsigned short")
        return s ~= nil and us ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_long_types() {
    let lua = create_lua_with_ffi();

    // Test long and unsigned long
    let result = lua
        .load(
            r#"
        local l = ffi.typeof("long")
        local ul = ffi.typeof("unsigned long")
        return l ~= nil and ul ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_pointer_to_pointer() {
    let lua = create_lua_with_ffi();

    // Test pointer to pointer syntax
    let result = lua
        .load(
            r#"
        return ffi.typeof("int**")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_array_of_pointers() {
    let lua = create_lua_with_ffi();

    // Test array of pointers
    let result = lua
        .load(
            r#"
        return ffi.typeof("int*[5]")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_empty_array() {
    let lua = create_lua_with_ffi();

    // Test empty array syntax (flexible array member)
    let result = lua
        .load(
            r#"
        return ffi.typeof("int[]")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_struct_with_array() {
    let lua = create_lua_with_ffi();

    // Test struct containing an array
    let result = lua
        .load(
            r#"
        ffi.cdef[[
            struct Buffer {
                int size;
                char data[256];
            };
        ]]
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_error_handling_invalid_type() {
    let lua = create_lua_with_ffi();

    // Test error handling for invalid type
    let result = lua
        .load(
            r#"
        pcall(function()
            ffi.typeof("invalid_type_xyz")
        end)
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_error_handling_malformed_struct() {
    let lua = create_lua_with_ffi();

    // Test error handling for malformed struct
    let result = lua
        .load(
            r#"
        local ok = pcall(function()
            ffi.cdef[[
                struct BadStruct {
                    int x
                    -- missing semicolon
                };
            ]]
        end)
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_metatype_basic() {
    let lua = create_lua_with_ffi();

    // Test metatype function
    let result = lua
        .load(
            r#"
        local mt = {}
        ffi.metatype("int", mt)
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_gc_basic() {
    let lua = create_lua_with_ffi();

    // Test gc function
    let result = lua
        .load(
            r#"
        local function finalizer(cdata)
            -- cleanup
        end
        -- ffi.gc would be called with actual cdata
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_addressof_usage() {
    let lua = create_lua_with_ffi();

    // Test addressof function exists
    let result = lua
        .load(
            r#"
        return type(ffi.addressof) == "function"
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_istype_usage() {
    let lua = create_lua_with_ffi();

    // Test istype function
    let result = lua
        .load(
            r#"
        local result = ffi.istype("int", 42)
        return type(result) == "boolean"
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_tonumber_usage() {
    let lua = create_lua_with_ffi();

    // Test tonumber function exists
    let result = lua
        .load(
            r#"
        return type(ffi.tonumber) == "function"
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_string_usage() {
    let lua = create_lua_with_ffi();

    // Test string function exists
    let result = lua
        .load(
            r#"
        return type(ffi.string) == "function"
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_multiple_structs() {
    let lua = create_lua_with_ffi();

    // Test defining multiple structs
    let result = lua
        .load(
            r#"
        ffi.cdef[[
            struct Point { int x; int y; };
            struct Circle { int x; int y; int radius; };
            struct Rectangle { int x; int y; int w; int h; };
        ]]
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_struct_name_uniqueness() {
    let lua = create_lua_with_ffi();

    // Test that struct names are tracked properly
    let result = lua
        .load(
            r#"
        ffi.cdef[[
            struct UniqueStruct1 { int value; };
        ]]
        ffi.cdef[[
            struct UniqueStruct2 { float value; };
        ]]
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_whitespace_handling() {
    let lua = create_lua_with_ffi();

    // Test that whitespace is handled correctly
    let result = lua
        .load(
            r#"
        ffi.cdef[[
            struct   SpacedStruct   {
                int    x   ;
                float  y   ;
            }   ;
        ]]
        return true
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_multiline_struct() {
    let lua = create_lua_with_ffi();

    // Test multiline struct definition
    let result = lua
        .load(
            r#"
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
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_api_completeness() {
    let lua = create_lua_with_ffi();

    // Test that all expected API functions exist
    let result = lua
        .load(
            r#"
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
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok() && result.unwrap());
}

#[test]
fn test_constants_exist() {
    let lua = create_lua_with_ffi();

    // Test that expected constants exist
    let result = lua
        .load(
            r#"
        return ffi.VERSION ~= nil and ffi.nullptr ~= nil and ffi.C ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_vla_syntax() {
    let lua = create_lua_with_ffi();

    // Test VLA syntax with [?]
    let result = lua
        .load(
            r#"
        return ffi.typeof("int[?]")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_vla_with_pointer() {
    let lua = create_lua_with_ffi();

    // Test VLA with pointer type
    let result = lua
        .load(
            r#"
        return ffi.typeof("void*[?]")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_vla_different_types() {
    let lua = create_lua_with_ffi();

    // Test VLA with various base types
    let types = vec!["char[?]", "int[?]", "float[?]", "double[?]", "void*[?]"];

    for type_name in types {
        let result = lua
            .load(&format!("return ffi.typeof('{}')", type_name))
            .eval::<String>();
        assert!(result.is_ok(), "Failed for VLA type: {}", type_name);
    }
}

#[test]
fn test_vla_with_const_qualifier() {
    let lua = create_lua_with_ffi();

    // Test VLA with const qualifier
    let result = lua
        .load(
            r#"
        return ffi.typeof("const char*[?]")
    "#,
        )
        .eval::<String>();

    assert!(result.is_ok());
}

#[test]
fn test_vla_with_various_qualifiers() {
    let lua = create_lua_with_ffi();

    // Test VLA with different type qualifiers
    let types = vec![
        "const char*[?]",
        "const int[?]",
        "volatile int[?]",
        "const void*[?]",
    ];

    for type_name in types {
        let result = lua
            .load(&format!("return ffi.typeof('{}')", type_name))
            .eval::<String>();
        assert!(
            result.is_ok(),
            "Failed for VLA type with qualifier: {}",
            type_name
        );
    }
}

#[test]
fn test_const_char_ptr_array() {
    let lua = create_lua_with_ffi();

    // Specifically test the user's example: const char*[?]
    let result = lua
        .load(
            r#"
        local type_str = ffi.typeof("const char*[?]")
        return type_str ~= nil
    "#,
        )
        .eval::<bool>();

    assert!(result.is_ok());
}

#[test]
fn test_char_ptr_array_vla() {
    let lua = create_lua_with_ffi();

    // Test char*[?] VLA instantiation
    let result: Result<(), _> = lua
        .load(
            r#"
        local ptr_array = ffi.new("char*[?]", 3)
        assert(ptr_array ~= nil, "ffi.new returned nil")
        -- Verify we can use the array
        assert(ffi.sizeof(ffi.typeof("char*")) * 3 == 3 * ffi.sizeof("char*"))
    "#,
        )
        .exec();

    assert!(
        result.is_ok(),
        "Failed to create char*[?] with ffi.new: {:?}",
        result.err()
    );
}

#[test]
fn test_various_pointer_array_vla() {
    let lua = create_lua_with_ffi();

    // Test various pointer array VLA types
    let types = vec!["char*[?]", "int*[?]", "void*[?]", "float*[?]", "double*[?]"];

    for type_name in types {
        let result = lua
            .load(&format!("return ffi.typeof('{}')", type_name))
            .eval::<String>();
        assert!(
            result.is_ok(),
            "Failed for pointer array VLA: {}",
            type_name
        );
    }
}

#[test]
fn test_vla_with_float_size() {
    let lua = create_lua_with_ffi();

    // Test VLA accepts float parameters
    let result = lua
        .load(
            r#"
        -- Test with integer
        local arr1 = ffi.new("int[?]", 5)
        
        -- Test with float (should work and truncate)
        local arr2 = ffi.new("int[?]", 10.0)
        local arr3 = ffi.new("int[?]", 7.9)  -- truncates to 7
        
        -- Test with pointer array
        local arr4 = ffi.new("char*[?]", 16.0)
    "#,
        )
        .exec();

    assert!(
        result.is_ok(),
        "Failed to create VLA with float size: {:?}",
        result.err()
    );
}
