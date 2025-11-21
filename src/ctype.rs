use std::mem::{align_of, size_of};

/// C type representation with size and alignment information
#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    // Basic types
    Bool,
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    LongLong,
    ULongLong,

    // Fixed-width integer types
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,

    // POSIX types (Unix only)
    #[cfg(unix)]
    InoT,
    #[cfg(unix)]
    DevT,
    #[cfg(unix)]
    GidT,
    #[cfg(unix)]
    ModeT,
    #[cfg(unix)]
    NlinkT,
    #[cfg(unix)]
    UidT,
    #[cfg(unix)]
    OffT,
    #[cfg(unix)]
    PidT,
    #[cfg(unix)]
    UsecondsT,
    #[cfg(unix)]
    SusecondsT,
    #[cfg(unix)]
    BlksizeT,
    #[cfg(unix)]
    BlkcntT,
    #[cfg(unix)]
    TimeT,
    
    // Common types across platforms
    SizeT,
    SSizeT,

    // Floating point
    Float,
    Double,

    // Complex types
    Void,
    Ptr(Box<CType>),
    Array(Box<CType>, usize),
    Struct(String, Vec<CField>),
    Union(String, Vec<CField>),
    Function(Box<CType>, Vec<CType>),
    Typedef(String, Box<CType>),
}

/// Struct/union field with name, type and offset
#[derive(Debug, Clone, PartialEq)]
pub struct CField {
    pub name: String,
    pub ctype: CType,
    pub offset: usize,
}

impl CType {
    /// Get the alignment requirement for this type
    #[inline]
    pub fn alignment(&self) -> usize {
        match self {
            CType::Bool => align_of::<bool>(),
            CType::Char | CType::UChar | CType::Int8 | CType::UInt8 => 1,
            CType::Short | CType::UShort | CType::Int16 | CType::UInt16 => 2,
            CType::Int | CType::UInt | CType::Int32 | CType::UInt32 | CType::Float => 4,
            CType::Long | CType::ULong | CType::LongLong | CType::ULongLong 
            | CType::Int64 | CType::UInt64 | CType::Double => 8,
            CType::SizeT | CType::SSizeT => align_of::<usize>(),
            CType::Void => 1,
            CType::Ptr(_) | CType::Function(_, _) => align_of::<*const ()>(),
            CType::Array(inner, _) | CType::Typedef(_, inner) => inner.alignment(),
            CType::Struct(_, fields) | CType::Union(_, fields) => fields
                .iter()
                .map(|f| f.ctype.alignment())
                .max()
                .unwrap_or(1),
            #[cfg(unix)]
            _ => 8,
        }
    }

    /// Get the size in bytes for this type
    #[inline]
    pub fn size(&self) -> usize {
        match self {
            CType::Bool => size_of::<bool>(),
            CType::Char | CType::UChar | CType::Int8 | CType::UInt8 => 1,
            CType::Short | CType::UShort | CType::Int16 | CType::UInt16 => 2,
            CType::Int | CType::UInt | CType::Int32 | CType::UInt32 => 4,
            CType::Long | CType::ULong => size_of::<isize>(),
            CType::LongLong | CType::ULongLong | CType::Int64 | CType::UInt64 => 8,
            CType::SizeT | CType::SSizeT => size_of::<usize>(),
            #[cfg(unix)]
            CType::InoT | CType::DevT | CType::GidT | CType::ModeT | CType::NlinkT 
            | CType::UidT | CType::OffT | CType::PidT | CType::UsecondsT 
            | CType::SusecondsT | CType::BlksizeT | CType::BlkcntT | CType::TimeT => 8,
            CType::Float => 4,
            CType::Double => 8,
            CType::Void => 0,
            CType::Ptr(_) | CType::Function(_, _) => size_of::<*const ()>(),
            CType::Array(inner, count) => inner.size() * count,
            CType::Struct(_, fields) => {
                if fields.is_empty() {
                    return 0;
                }
                // Find the maximum end offset
                let max_end = fields
                    .iter()
                    .map(|f| f.offset + f.ctype.size())
                    .max()
                    .unwrap_or(0);
                // Align to struct alignment using bit manipulation (faster)
                let align = self.alignment();
                (max_end + align - 1) & !(align - 1)
            }
            CType::Union(_, fields) => fields.iter().map(|f| f.ctype.size()).max().unwrap_or(0),
            CType::Typedef(_, inner) => inner.size(),
        }
    }
}
