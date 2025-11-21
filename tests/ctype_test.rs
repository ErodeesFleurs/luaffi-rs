use luaffi::ctype::{CType, CField};

#[test]
fn test_basic_type_sizes() {
    assert_eq!(CType::Bool.size(), std::mem::size_of::<bool>());
    assert_eq!(CType::Char.size(), std::mem::size_of::<u8>());
    assert_eq!(CType::Short.size(), std::mem::size_of::<i16>());
    assert_eq!(CType::Int.size(), std::mem::size_of::<i32>());
    assert_eq!(CType::Long.size(), std::mem::size_of::<isize>());
    assert_eq!(CType::Float.size(), std::mem::size_of::<f32>());
    assert_eq!(CType::Double.size(), std::mem::size_of::<f64>());
}

#[test]
fn test_fixed_width_type_sizes() {
    assert_eq!(CType::Int8.size(), 1);
    assert_eq!(CType::Int16.size(), 2);
    assert_eq!(CType::Int32.size(), 4);
    assert_eq!(CType::Int64.size(), 8);
    assert_eq!(CType::UInt8.size(), 1);
    assert_eq!(CType::UInt16.size(), 2);
    assert_eq!(CType::UInt32.size(), 4);
    assert_eq!(CType::UInt64.size(), 8);
}

#[test]
fn test_pointer_size() {
    let ptr_type = CType::Ptr(Box::new(CType::Int));
    assert_eq!(ptr_type.size(), std::mem::size_of::<*const ()>());
}

#[test]
fn test_array_size() {
    let array_type = CType::Array(Box::new(CType::Int), 10);
    assert_eq!(array_type.size(), std::mem::size_of::<i32>() * 10);
}

#[test]
fn test_void_size() {
    assert_eq!(CType::Void.size(), 0);
}

#[test]
fn test_basic_type_alignment() {
    assert_eq!(CType::Bool.alignment(), std::mem::align_of::<bool>());
    assert_eq!(CType::Char.alignment(), std::mem::align_of::<u8>());
    assert_eq!(CType::Short.alignment(), std::mem::align_of::<i16>());
    assert_eq!(CType::Int.alignment(), std::mem::align_of::<i32>());
    assert_eq!(CType::Long.alignment(), std::mem::align_of::<isize>());
    assert_eq!(CType::Float.alignment(), std::mem::align_of::<f32>());
    assert_eq!(CType::Double.alignment(), std::mem::align_of::<f64>());
}

#[test]
fn test_fixed_width_alignment() {
    assert_eq!(CType::Int8.alignment(), 1);
    assert_eq!(CType::Int16.alignment(), 2);
    assert_eq!(CType::Int32.alignment(), 4);
    assert_eq!(CType::Int64.alignment(), 8);
}

#[test]
fn test_pointer_alignment() {
    let ptr_type = CType::Ptr(Box::new(CType::Int));
    assert_eq!(ptr_type.alignment(), std::mem::align_of::<*const ()>());
}

#[test]
fn test_array_alignment() {
    let array_type = CType::Array(Box::new(CType::Int), 10);
    assert_eq!(array_type.alignment(), std::mem::align_of::<i32>());
}

#[test]
fn test_struct_size_empty() {
    let struct_type = CType::Struct("Empty".to_string(), vec![]);
    assert_eq!(struct_type.size(), 0);
}

#[test]
fn test_struct_size_single_field() {
    let fields = vec![
        CField {
            name: "x".to_string(),
            ctype: CType::Int,
            offset: 0,
        }
    ];
    let struct_type = CType::Struct("Single".to_string(), fields);
    assert!(struct_type.size() >= std::mem::size_of::<i32>());
}

#[test]
fn test_struct_size_multiple_fields() {
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
        }
    ];
    let struct_type = CType::Struct("Point".to_string(), fields);
    assert!(struct_type.size() >= std::mem::size_of::<i32>() * 2);
}

#[test]
fn test_union_size() {
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
        }
    ];
    let union_type = CType::Union("Value".to_string(), fields);
    // Union size is the max of all field sizes
    assert_eq!(union_type.size(), std::mem::size_of::<i32>().max(std::mem::size_of::<f32>()));
}

#[test]
fn test_nested_pointer() {
    let inner = CType::Int;
    let ptr1 = CType::Ptr(Box::new(inner));
    let ptr2 = CType::Ptr(Box::new(ptr1));
    assert_eq!(ptr2.size(), std::mem::size_of::<*const ()>());
}

#[test]
fn test_array_of_pointers() {
    let ptr_type = CType::Ptr(Box::new(CType::Int));
    let array_type = CType::Array(Box::new(ptr_type), 5);
    assert_eq!(array_type.size(), std::mem::size_of::<*const ()>() * 5);
}

#[test]
fn test_pointer_to_array() {
    let array_type = CType::Array(Box::new(CType::Int), 10);
    let ptr_type = CType::Ptr(Box::new(array_type));
    assert_eq!(ptr_type.size(), std::mem::size_of::<*const ()>());
}

#[test]
fn test_typedef_size() {
    let typedef = CType::Typedef("MyInt".to_string(), Box::new(CType::Int));
    assert_eq!(typedef.size(), std::mem::size_of::<i32>());
}

#[test]
fn test_typedef_alignment() {
    let typedef = CType::Typedef("MyInt".to_string(), Box::new(CType::Int));
    assert_eq!(typedef.alignment(), std::mem::align_of::<i32>());
}

#[test]
fn test_struct_alignment() {
    let fields = vec![
        CField {
            name: "c".to_string(),
            ctype: CType::Char,
            offset: 0,
        },
        CField {
            name: "i".to_string(),
            ctype: CType::Int,
            offset: 4,
        }
    ];
    let struct_type = CType::Struct("Mixed".to_string(), fields);
    // Struct alignment should be the max of all field alignments
    assert_eq!(struct_type.alignment(), std::mem::align_of::<i32>());
}

#[test]
fn test_union_alignment() {
    let fields = vec![
        CField {
            name: "c".to_string(),
            ctype: CType::Char,
            offset: 0,
        },
        CField {
            name: "d".to_string(),
            ctype: CType::Double,
            offset: 0,
        }
    ];
    let union_type = CType::Union("MixedUnion".to_string(), fields);
    // Union alignment should be the max of all field alignments
    assert_eq!(union_type.alignment(), std::mem::align_of::<f64>());
}

#[test]
fn test_size_t_size() {
    assert_eq!(CType::SizeT.size(), std::mem::size_of::<usize>());
}

#[test]
fn test_ssize_t_size() {
    assert_eq!(CType::SSizeT.size(), std::mem::size_of::<usize>());
}

#[test]
fn test_ctype_equality() {
    assert_eq!(CType::Int, CType::Int);
    assert_eq!(CType::Float, CType::Float);
    assert_ne!(CType::Int, CType::Float);
}

#[test]
fn test_pointer_equality() {
    let ptr1 = CType::Ptr(Box::new(CType::Int));
    let ptr2 = CType::Ptr(Box::new(CType::Int));
    let ptr3 = CType::Ptr(Box::new(CType::Float));
    
    assert_eq!(ptr1, ptr2);
    assert_ne!(ptr1, ptr3);
}

#[test]
fn test_array_equality() {
    let arr1 = CType::Array(Box::new(CType::Int), 10);
    let arr2 = CType::Array(Box::new(CType::Int), 10);
    let arr3 = CType::Array(Box::new(CType::Int), 20);
    
    assert_eq!(arr1, arr2);
    assert_ne!(arr1, arr3);
}

#[test]
fn test_empty_array() {
    let array_type = CType::Array(Box::new(CType::Int), 0);
    assert_eq!(array_type.size(), 0);
}

#[test]
fn test_large_array() {
    let array_type = CType::Array(Box::new(CType::Char), 1024);
    assert_eq!(array_type.size(), 1024);
}

#[test]
fn test_multi_dimensional_array() {
    let inner_array = CType::Array(Box::new(CType::Int), 10);
    let outer_array = CType::Array(Box::new(inner_array.clone()), 5);
    
    // 5 arrays of 10 ints each
    assert_eq!(outer_array.size(), std::mem::size_of::<i32>() * 10 * 5);
}

#[test]
fn test_cfield_clone() {
    let field = CField {
        name: "test".to_string(),
        ctype: CType::Int,
        offset: 4,
    };
    
    let cloned = field.clone();
    assert_eq!(field.name, cloned.name);
    assert_eq!(field.ctype, cloned.ctype);
    assert_eq!(field.offset, cloned.offset);
}

#[test]
fn test_complex_struct() {
    let fields = vec![
        CField {
            name: "a".to_string(),
            ctype: CType::Char,
            offset: 0,
        },
        CField {
            name: "b".to_string(),
            ctype: CType::Int,
            offset: 4,
        },
        CField {
            name: "c".to_string(),
            ctype: CType::Double,
            offset: 8,
        },
        CField {
            name: "d".to_string(),
            ctype: CType::Ptr(Box::new(CType::Char)),
            offset: 16,
        }
    ];
    
    let struct_type = CType::Struct("Complex".to_string(), fields);
    assert!(struct_type.size() > 0);
    assert!(struct_type.alignment() > 0);
}
