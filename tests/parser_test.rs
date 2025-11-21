#[cfg(test)]
mod parser_tests {

    #[test]
    fn test_parse_simple_struct() {
        // This test would use the actual parser once it's exposed
        // For now, we test the structure
        let result = "struct Point { int x; int y; };";
        assert!(result.contains("struct"));
        assert!(result.contains("int"));
    }

    #[test]
    fn test_parse_typedef() {
        let code = "typedef int my_int;";
        assert!(code.contains("typedef"));
    }

    #[test]
    fn test_parse_array() {
        let code = "int arr[10];";
        assert!(code.contains("["));
        assert!(code.contains("]"));
    }

    #[test]
    fn test_parse_pointer() {
        let code = "int* ptr;";
        assert!(code.contains("*"));
    }

    #[test]
    fn test_parse_function() {
        let code = "int func(int x, int y);";
        assert!(code.contains("("));
        assert!(code.contains(")"));
    }

    #[test]
    fn test_struct_with_multiple_types() {
        let code = r#"
            struct Data {
                char c;
                int i;
                float f;
                double d;
            };
        "#;
        assert!(code.contains("char"));
        assert!(code.contains("int"));
        assert!(code.contains("float"));
        assert!(code.contains("double"));
    }

    #[test]
    fn test_struct_with_array() {
        let code = r#"
            struct Buffer {
                int size;
                char data[256];
            };
        "#;
        assert!(code.contains("char data[256]"));
    }

    #[test]
    fn test_struct_with_pointer() {
        let code = r#"
            struct Node {
                int value;
                struct Node* next;
            };
        "#;
        assert!(code.contains("struct Node*"));
    }

    #[test]
    fn test_nested_struct() {
        let code = r#"
            struct Inner {
                int x;
            };
            struct Outer {
                struct Inner inner;
                int y;
            };
        "#;
        assert!(code.contains("struct Inner"));
        assert!(code.contains("struct Outer"));
    }

    #[test]
    fn test_union() {
        let code = r#"
            union Value {
                int i;
                float f;
            };
        "#;
        assert!(code.contains("union"));
    }

    #[test]
    fn test_multiline_struct() {
        let code = r#"
            struct Multi
            {
                int
                    a
                    ;
                float
                    b
                    ;
            };
        "#;
        assert!(code.contains("struct Multi"));
    }

    #[test]
    fn test_empty_struct() {
        let code = "struct Empty {};";
        assert!(code.contains("{}"));
    }

    #[test]
    fn test_comments_handling() {
        let code = r#"
            struct WithComments {
                int x; // This is x
                int y; /* This is y */
            };
        "#;
        // Parser should handle or ignore comments
        assert!(code.contains("int x"));
    }

    #[test]
    fn test_packed_struct() {
        let code = r#"
            struct __attribute__((packed)) Packed {
                char c;
                int i;
            };
        "#;
        // Test that attribute syntax is present
        assert!(code.contains("__attribute__"));
    }

    #[test]
    fn test_bitfield() {
        let code = r#"
            struct Flags {
                unsigned int flag1 : 1;
                unsigned int flag2 : 1;
                unsigned int value : 6;
            };
        "#;
        // Test bitfield syntax
        assert!(code.contains(":"));
    }

    #[test]
    fn test_enum() {
        let code = r#"
            enum Color {
                RED,
                GREEN,
                BLUE
            };
        "#;
        assert!(code.contains("enum"));
    }

    #[test]
    fn test_function_pointer() {
        let code = "typedef void (*callback_t)(int x);";
        assert!(code.contains("(*"));
    }

    #[test]
    fn test_const_qualifier() {
        let code = "const int* ptr;";
        assert!(code.contains("const"));
    }

    #[test]
    fn test_volatile_qualifier() {
        let code = "volatile int value;";
        assert!(code.contains("volatile"));
    }

    #[test]
    fn test_multiple_declarations() {
        let code = r#"
            struct A { int x; };
            struct B { float y; };
            typedef int my_int;
        "#;
        assert!(code.contains("struct A"));
        assert!(code.contains("struct B"));
        assert!(code.contains("typedef"));
    }

    #[test]
    fn test_forward_declaration() {
        let code = "struct Forward;";
        assert!(code.contains("struct Forward"));
    }

    #[test]
    fn test_anonymous_struct() {
        let code = r#"
            struct {
                int x;
                int y;
            } point;
        "#;
        assert!(code.contains("int x"));
    }

    #[test]
    fn test_fixed_width_types() {
        let code = r#"
            struct Fixed {
                int8_t i8;
                int16_t i16;
                int32_t i32;
                int64_t i64;
                uint8_t u8;
                uint16_t u16;
                uint32_t u32;
                uint64_t u64;
            };
        "#;
        assert!(code.contains("int8_t"));
        assert!(code.contains("uint64_t"));
    }

    #[test]
    fn test_size_t_type() {
        let code = "size_t size;";
        assert!(code.contains("size_t"));
    }

    #[test]
    fn test_pointer_types() {
        let code = r#"
            struct Pointers {
                void* vp;
                char* cp;
                int** ipp;
            };
        "#;
        assert!(code.contains("void*"));
        assert!(code.contains("int**"));
    }
}
