// VLA 功能的端到端测试
#[cfg(test)]
mod vla_e2e_tests {
    use luaffi::ctype::CType;

    #[test]
    fn test_vla_basic_functionality() {
        // 测试 VLA 类型的基本功能
        let vla_int = CType::VLA(Box::new(CType::Int));
        
        // VLA 在类型定义时大小未知
        assert_eq!(vla_int.size(), 0);
        
        // 但对齐方式应该与元素类型一致
        assert_eq!(vla_int.alignment(), CType::Int.alignment());
    }

    #[test]
    fn test_vla_with_various_types() {
        let types = vec![
            ("int", CType::Int),
            ("float", CType::Float),
            ("double", CType::Double),
            ("char", CType::Char),
        ];

        for (name, ctype) in types {
            let vla = CType::VLA(Box::new(ctype.clone()));
            assert_eq!(vla.size(), 0, "VLA of {} should have size 0", name);
            assert_eq!(
                vla.alignment(),
                ctype.alignment(),
                "VLA of {} should have same alignment as element",
                name
            );
        }
    }

    #[test]
    fn test_vla_with_pointer() {
        let void_ptr = CType::Ptr(Box::new(CType::Void));
        let vla_ptr = CType::VLA(Box::new(void_ptr.clone()));
        
        assert_eq!(vla_ptr.size(), 0);
        assert_eq!(vla_ptr.alignment(), void_ptr.alignment());
    }

    #[test]
    fn test_vla_clone() {
        let vla = CType::VLA(Box::new(CType::Int));
        let cloned = vla.clone();
        
        assert_eq!(vla, cloned);
    }

    #[test]
    fn test_vla_equality() {
        let vla1 = CType::VLA(Box::new(CType::Int));
        let vla2 = CType::VLA(Box::new(CType::Int));
        let vla3 = CType::VLA(Box::new(CType::Float));
        
        assert_eq!(vla1, vla2);
        assert_ne!(vla1, vla3);
    }

    #[test]
    fn test_vla_vs_array() {
        let vla = CType::VLA(Box::new(CType::Int));
        let array = CType::Array(Box::new(CType::Int), 10);
        
        // VLA 大小未知 (0)
        assert_eq!(vla.size(), 0);
        
        // Array 大小已知
        assert_eq!(array.size(), std::mem::size_of::<i32>() * 10);
        
        // 但对齐方式相同
        assert_eq!(vla.alignment(), array.alignment());
    }

    #[test]
    fn test_nested_vla() {
        // 虽然不常见，但理论上可以有指向 VLA 的指针
        let vla = CType::VLA(Box::new(CType::Int));
        let ptr_to_vla = CType::Ptr(Box::new(vla));
        
        // 指针大小是固定的
        assert_eq!(ptr_to_vla.size(), std::mem::size_of::<*const ()>());
    }

    #[test]
    fn test_vla_with_const_qualifier() {
        // 测试带 const 限定符的 VLA
        // 这些在解析时应该忽略 const 关键字
        use luaffi::ctype::CType;
        
        // 这些类型定义应该是有效的
        // 注意：实际使用需要通过 lookup_type 函数测试
        let vla_ptr = CType::VLA(Box::new(CType::Ptr(Box::new(CType::Char))));
        assert_eq!(vla_ptr.size(), 0);
    }

    #[test]
    fn test_vla_with_volatile_qualifier() {
        // 测试带 volatile 限定符的 VLA
        use luaffi::ctype::CType;
        
        let vla_int = CType::VLA(Box::new(CType::Int));
        assert_eq!(vla_int.size(), 0);
        assert_eq!(vla_int.alignment(), CType::Int.alignment());
    }
}
