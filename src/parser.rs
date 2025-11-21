use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{char, digit1, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list0};
use nom::sequence::delimited;

use crate::ctype::{CField, CType};
use crate::ffi_ops;

/// Parse C definitions and register types in the global registry
pub fn parse_cdef(code: &str) -> Result<(), String> {
    let result: IResult<&str, Vec<()>> = many0(parse_declaration).parse(code);

    match result {
        Ok((remaining, _)) => {
            let trimmed = remaining.trim();
            if trimmed.is_empty() {
                Ok(())
            } else {
                Err(format!("Unparsed input remaining ({}): '{}'", 
                    trimmed.len(), 
                    trimmed.chars().take(50).collect::<String>()
                ))
            }
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

/// Parse a single declaration (struct, typedef, or function)
fn parse_declaration(input: &str) -> IResult<&str, ()> {
    let (input, _) = multispace0(input)?;
    
    // Early return if no input left
    if input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));
    }
    
    // Try parsing different declaration types
    alt((
        map(parse_struct, |_| ()),
        map(parse_typedef, |_| ()),
        map(parse_function, |_| ()),
    )).parse(input)
}

fn parse_struct(input: &str) -> IResult<&str, CType> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("struct")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, mut fields) = delimited(char('{'), parse_struct_fields, char('}')).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(';')(input)?;
    let (input, _) = multispace0(input)?;

    // Calculate field offsets with proper alignment
    calculate_field_offsets(&mut fields);

    let name_string = name.to_string();
    let ctype = CType::Struct(name_string.clone(), fields);
    
    // Register the type in global registry
    ffi_ops::register_type(name_string, ctype.clone());

    Ok((input, ctype))
}

/// Calculate field offsets with proper alignment
#[inline]
fn calculate_field_offsets(fields: &mut [CField]) {
    let mut offset = 0;
    for field in fields.iter_mut() {
        let align = field.ctype.alignment();
        // Align offset to field alignment
        offset = (offset + align - 1) & !(align - 1);
        field.offset = offset;
        offset += field.ctype.size();
    }
}

fn parse_struct_fields(input: &str) -> IResult<&str, Vec<CField>> {
    let (input, _) = multispace0(input)?;
    let (input, fields) = separated_list0(char(';'), parse_field).parse(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, fields))
}

fn parse_field(input: &str) -> IResult<&str, CField> {
    let (input, _) = multispace0(input)?;
    let (input, type_name) = parse_type(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = identifier(input)?;
    let (input, array_size) = opt(parse_array_size).parse(input)?;
    let (input, _) = multispace0(input)?;

    let ctype = if let Some(size) = array_size {
        CType::Array(Box::new(type_name), size)
    } else {
        type_name
    };

    Ok((
        input,
        CField {
            name: name.to_string(),
            ctype,
            offset: 0, // Will be calculated later
        },
    ))
}

// Parse type with optimized matching - use ffi_ops lookup to avoid duplication
fn parse_type(input: &str) -> IResult<&str, CType> {
    let (input, type_str) = identifier(input)?;

    // Try to look up as basic type first (fast path)
    let ctype = if let Ok(basic_type) = ffi_ops::lookup_type(type_str) {
        basic_type
    } else {
        // Fall back to typedef for unknown types
        CType::Typedef(type_str.to_string(), Box::new(CType::Int))
    };

    Ok((input, ctype))
}

fn parse_array_size(input: &str) -> IResult<&str, usize> {
    let (input, _) = char('[')(input)?;
    let (input, digits) = digit1(input)?;
    let (input, _) = char(']')(input)?;
    let size = digits.parse().expect("Failed to parse array size");
    Ok((input, size))
}

fn parse_typedef(input: &str) -> IResult<&str, ()> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("typedef")(input)?;
    let (input, _) = multispace1(input)?;
    // Skip typedef for now
    let (input, _) = take_while(|c| c != ';')(input)?;
    let (input, _) = char(';')(input)?;
    Ok((input, ()))
}

fn parse_function(input: &str) -> IResult<&str, ()> {
    // Skip function declarations for now
    // Must consume at least one character
    let (input, _) = take_while1(|c: char| c != ';' && c != '\n')(input)?;
    let (input, _) = opt(char(';')).parse(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ()))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_struct() {
        let code = "struct Point { int x; int y; };";
        let result = parse_cdef(code);
        if let Err(e) = &result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
    }
}
