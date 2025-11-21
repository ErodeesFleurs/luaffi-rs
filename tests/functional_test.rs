// Functional tests for FFI operations
// These tests check the actual implementation behavior

#[cfg(test)]
mod ffi_functional_tests {
    use luaffi::ctype::{CType, CField};

    #[test]
    fn test_create_basic_types() {
        let types = vec![
            CType::Bool,
            CType::Char,
            CType::UChar,
            CType::Short,
            CType::UShort,
            CType::Int,
            CType::UInt,
            CType::Long,
            CType::ULong,
            CType::Float,
            CType::Double,
            CType::Void,
        ];

        for t in types {
            // Just verify we can create and query basic properties
            let _ = t.size();
            let _ = t.alignment();
        }
    }

    #[test]
    fn test_create_pointer_types() {
        let int_ptr = CType::Ptr(Box::new(CType::Int));
        assert_eq!(int_ptr.size(), std::mem::size_of::<*const ()>());

        let char_ptr = CType::Ptr(Box::new(CType::Char));
        assert_eq!(char_ptr.size(), std::mem::size_of::<*const ()>());

        // Double pointer
        let ptr_ptr = CType::Ptr(Box::new(int_ptr.clone()));
        assert_eq!(ptr_ptr.size(), std::mem::size_of::<*const ()>());
    }

    #[test]
    fn test_create_array_types() {
        let arr = CType::Array(Box::new(CType::Int), 10);
        assert_eq!(arr.size(), std::mem::size_of::<i32>() * 10);

        let arr2 = CType::Array(Box::new(CType::Char), 256);
        assert_eq!(arr2.size(), 256);

        // Empty array
        let arr3 = CType::Array(Box::new(CType::Int), 0);
        assert_eq!(arr3.size(), 0);
    }

    #[test]
    fn test_create_struct_type() {
        let fields = vec![
            CField {
                name: "x".to_string(),
                ctype: CType::Int,
                offset: 0,
            },
            CField {
                name: "y".to_string(),
                ctype: CType::Int,
                offset: 4,
            },
        ];

        let point_struct = CType::Struct("Point".to_string(), fields);
        assert!(point_struct.size() >= 8);
        assert!(point_struct.alignment() >= std::mem::align_of::<i32>());
    }

    #[test]
    fn test_create_union_type() {
        let fields = vec![
            CField {
                name: "i".to_string(),
                ctype: CType::Int,
                offset: 0,
            },
            CField {
                name: "f".to_string(),
                ctype: CType::Float,
                offset: 0,
            },
        ];

        let value_union = CType::Union("Value".to_string(), fields);
        assert_eq!(value_union.size(), std::mem::size_of::<i32>());
    }

    #[test]
    fn test_struct_with_different_alignment() {
        let fields = vec![
            CField {
                name: "c".to_string(),
                ctype: CType::Char,
                offset: 0,
            },
            CField {
                name: "d".to_string(),
                ctype: CType::Double,
                offset: 8,
            },
        ];

        let mixed_struct = CType::Struct("Mixed".to_string(), fields);
        // Should be aligned to double's alignment
        assert_eq!(mixed_struct.alignment(), std::mem::align_of::<f64>());
    }

    #[test]
    fn test_nested_struct_types() {
        let inner_fields = vec![
            CField {
                name: "x".to_string(),
                ctype: CType::Int,
                offset: 0,
            },
        ];
        let inner = CType::Struct("Inner".to_string(), inner_fields);

        let outer_fields = vec![
            CField {
                name: "inner".to_string(),
                ctype: inner.clone(),
                offset: 0,
            },
            CField {
                name: "y".to_string(),
                ctype: CType::Int,
                offset: inner.size(),
            },
        ];

        let outer = CType::Struct("Outer".to_string(), outer_fields);
        assert!(outer.size() >= inner.size() + std::mem::size_of::<i32>());
    }

    #[test]
    fn test_struct_with_array_field() {
        let arr = CType::Array(Box::new(CType::Char), 256);
        let fields = vec![
            CField {
                name: "size".to_string(),
                ctype: CType::Int,
                offset: 0,
            },
            CField {
                name: "data".to_string(),
                ctype: arr.clone(),
                offset: 4,
            },
        ];

        let buffer = CType::Struct("Buffer".to_string(), fields);
        assert!(buffer.size() >= 256 + std::mem::size_of::<i32>());
    }

    #[test]
    fn test_struct_with_pointer_field() {
        let ptr = CType::Ptr(Box::new(CType::Char));
        let fields = vec![
            CField {
                name: "data".to_string(),
                ctype: ptr,
                offset: 0,
            },
            CField {
                name: "size".to_string(),
                ctype: CType::Int,
                offset: std::mem::size_of::<*const ()>(),
            },
        ];

        let string_view = CType::Struct("StringView".to_string(), fields);
        assert!(string_view.size() >= std::mem::size_of::<*const ()>() + std::mem::size_of::<i32>());
    }

    #[test]
    fn test_typedef_basic() {
        let my_int = CType::Typedef("my_int".to_string(), Box::new(CType::Int));
        assert_eq!(my_int.size(), std::mem::size_of::<i32>());
        assert_eq!(my_int.alignment(), std::mem::align_of::<i32>());
    }

    #[test]
    fn test_typedef_pointer() {
        let ptr = CType::Ptr(Box::new(CType::Char));
        let string_ptr = CType::Typedef("string_ptr".to_string(), Box::new(ptr));
        assert_eq!(string_ptr.size(), std::mem::size_of::<*const ()>());
    }

    #[test]
    fn test_typedef_struct() {
        let fields = vec![
            CField {
                name: "x".to_string(),
                ctype: CType::Int,
                offset: 0,
            },
        ];
        let point = CType::Struct("Point".to_string(), fields);
        let point_t = CType::Typedef("point_t".to_string(), Box::new(point.clone()));
        assert_eq!(point_t.size(), point.size());
    }

    #[test]
    fn test_array_of_structs() {
        let fields = vec![
            CField {
                name: "x".to_string(),
                ctype: CType::Int,
                offset: 0,
            },
            CField {
                name: "y".to_string(),
                ctype: CType::Int,
                offset: 4,
            },
        ];
        let point = CType::Struct("Point".to_string(), fields);
        let points_array = CType::Array(Box::new(point.clone()), 10);
        
        assert_eq!(points_array.size(), point.size() * 10);
    }

    #[test]
    fn test_pointer_to_struct() {
        let fields = vec![
            CField {
                name: "value".to_string(),
                ctype: CType::Int,
                offset: 0,
            },
        ];
        let node = CType::Struct("Node".to_string(), fields);
        let node_ptr = CType::Ptr(Box::new(node));
        
        assert_eq!(node_ptr.size(), std::mem::size_of::<*const ()>());
    }

    #[test]
    fn test_function_type() {
        let callback = CType::Function(
            Box::new(CType::Void),
            vec![CType::Int, CType::Int],
        );
        
        // Function types are stored as pointers
        assert_eq!(callback.size(), std::mem::size_of::<*const ()>());
    }

    #[test]
    fn test_complex_nested_structure() {
        // Create a complex structure with multiple levels of nesting
        let int_array = CType::Array(Box::new(CType::Int), 5);
        let char_ptr = CType::Ptr(Box::new(CType::Char));
        
        let inner_fields = vec![
            CField {
                name: "id".to_string(),
                ctype: CType::Int,
                offset: 0,
            },
            CField {
                name: "name".to_string(),
                ctype: char_ptr.clone(),
                offset: 4,
            },
        ];
        let inner = CType::Struct("Inner".to_string(), inner_fields);
        
        let outer_fields = vec![
            CField {
                name: "inner".to_string(),
                ctype: inner.clone(),
                offset: 0,
            },
            CField {
                name: "values".to_string(),
                ctype: int_array.clone(),
                offset: inner.size(),
            },
            CField {
                name: "count".to_string(),
                ctype: CType::SizeT,
                offset: inner.size() + int_array.size(),
            },
        ];
        let outer = CType::Struct("Outer".to_string(), outer_fields);
        
        assert!(outer.size() > 0);
        assert!(outer.alignment() > 0);
    }

    #[test]
    fn test_type_cloning() {
        let original = CType::Int;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let ptr_original = CType::Ptr(Box::new(CType::Int));
        let ptr_cloned = ptr_original.clone();
        assert_eq!(ptr_original, ptr_cloned);
    }

    #[test]
    fn test_field_offset_calculation() {
        // Test that we can work with field offsets
        let fields = vec![
            CField {
                name: "a".to_string(),
                ctype: CType::Char,
                offset: 0,
            },
            CField {
                name: "b".to_string(),
                ctype: CType::Int,
                offset: 4, // Aligned to 4 bytes
            },
            CField {
                name: "c".to_string(),
                ctype: CType::Double,
                offset: 8, // Aligned to 8 bytes
            },
        ];

        let _s = CType::Struct("Aligned".to_string(), fields.clone());
        
        // Verify offsets are reasonable
        assert_eq!(fields[0].offset, 0);
        assert!(fields[1].offset >= 4);
        assert!(fields[2].offset >= 8);
    }

    #[test]
    fn test_zero_sized_types() {
        let void_type = CType::Void;
        assert_eq!(void_type.size(), 0);

        let empty_array = CType::Array(Box::new(CType::Int), 0);
        assert_eq!(empty_array.size(), 0);

        let empty_struct = CType::Struct("Empty".to_string(), vec![]);
        assert_eq!(empty_struct.size(), 0);
    }

    #[test]
    fn test_all_basic_types_have_size() {
        let types = vec![
            CType::Bool, CType::Char, CType::UChar,
            CType::Short, CType::UShort,
            CType::Int, CType::UInt,
            CType::Long, CType::ULong,
            CType::Float, CType::Double,
            CType::Int8, CType::Int16, CType::Int32, CType::Int64,
            CType::UInt8, CType::UInt16, CType::UInt32, CType::UInt64,
            CType::SizeT, CType::SSizeT,
        ];

        for t in types {
            assert!(t.size() > 0, "Type {:?} should have size > 0", t);
            assert!(t.alignment() > 0, "Type {:?} should have alignment > 0", t);
        }
    }

    #[test]
    fn test_posix_types() {
        let posix_types = vec![
            CType::TimeT,
            CType::InoT,
            CType::DevT,
            CType::GidT,
            CType::UidT,
            CType::PidT,
            CType::OffT,
            CType::ModeT,
            CType::NlinkT,
        ];

        for t in posix_types {
            assert!(t.size() > 0);
            assert!(t.alignment() > 0);
        }
    }

    #[test]
    fn test_vla_type_creation() {
        // Test VLA type creation
        let vla = CType::VLA(Box::new(CType::Int));
        
        // VLA size should be 0 at definition time (size unknown)
        assert_eq!(vla.size(), 0);
        
        // VLA alignment should match element type
        assert_eq!(vla.alignment(), CType::Int.alignment());
    }

    #[test]
    fn test_vla_with_pointer() {
        let ptr_type = CType::Ptr(Box::new(CType::Void));
        let vla = CType::VLA(Box::new(ptr_type));
        
        assert_eq!(vla.size(), 0);
        assert_eq!(vla.alignment(), std::mem::align_of::<*const ()>());
    }

    #[test]
    fn test_vla_element_types() {
        let types = vec![
            CType::Char,
            CType::Int,
            CType::Float,
            CType::Double,
            CType::Ptr(Box::new(CType::Void)),
        ];
        
        for elem_type in types {
            let vla = CType::VLA(Box::new(elem_type.clone()));
            assert_eq!(vla.size(), 0);
            assert_eq!(vla.alignment(), elem_type.alignment());
        }
    }

    #[test]
    fn test_vla_with_const_pointer() {
        // Test VLA of const char* (common use case for string arrays)
        let char_type = CType::Char;
        let ptr_type = CType::Ptr(Box::new(char_type));
        let vla = CType::VLA(Box::new(ptr_type.clone()));
        
        assert_eq!(vla.size(), 0);
        assert_eq!(vla.alignment(), ptr_type.alignment());
    }

    #[test]
    fn test_array_of_const_pointers() {
        // Test creating an array of const char* pointers
        let char_type = CType::Char;
        let ptr_type = CType::Ptr(Box::new(char_type));
        let array = CType::Array(Box::new(ptr_type.clone()), 10);
        
        assert_eq!(array.size(), std::mem::size_of::<*const ()>() * 10);
        assert_eq!(array.alignment(), ptr_type.alignment());
    }

    #[test]
    fn test_vla_of_pointers() {
        // Test VLA of pointer types (e.g., char*[?])
        let char_type = CType::Char;
        let ptr_type = CType::Ptr(Box::new(char_type));
        let vla = CType::VLA(Box::new(ptr_type.clone()));
        
        assert_eq!(vla.size(), 0); // VLA size unknown at type definition
        assert_eq!(vla.alignment(), ptr_type.alignment());
    }

    #[test]
    fn test_vla_various_pointer_types() {
        // Test VLA with different pointer types
        let types = vec![
            CType::Ptr(Box::new(CType::Char)),
            CType::Ptr(Box::new(CType::Int)),
            CType::Ptr(Box::new(CType::Void)),
            CType::Ptr(Box::new(CType::Float)),
        ];
        
        for ptr_type in types {
            let vla = CType::VLA(Box::new(ptr_type.clone()));
            assert_eq!(vla.size(), 0);
            assert_eq!(vla.alignment(), ptr_type.alignment());
        }
    }
}
