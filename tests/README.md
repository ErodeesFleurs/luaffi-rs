# LuaFFI Test Suite

This directory contains comprehensive tests for the LuaFFI library.

## Test Files

### `ctype_test.rs`

Tests for the C type system implementation:

- Basic type sizes and alignments
- Fixed-width integer types (int8_t, uint32_t, etc.)
- Pointer types
- Array types
- Struct types (empty, single field, multiple fields)
- Union types
- Nested structures
- Typedef handling
- Complex compositions

### `functional_test.rs`

Functional tests for FFI operations:

- Creating and manipulating C types
- Type size and alignment calculations
- Nested structures
- Array of structs
- Pointers to structs
- Function types
- Field offset calculations
- Zero-sized types
- POSIX types

### `parser_test.rs`

Tests for the C declaration parser:

- Simple struct parsing
- Typedef parsing
- Array syntax
- Pointer syntax
- Function declarations
- Multi-line structures
- Nested structures
- Union declarations
- Forward declarations
- Fixed-width types
- Comments handling

### `integration_test.rs`

Integration tests for the complete FFI module:

- Module loading
- Basic type operations
- `ffi.cdef()` functionality
- `ffi.new()` allocation
- `ffi.cast()` type casting
- `ffi.sizeof()` queries
- `ffi.offsetof()` field offset queries
- `ffi.typeof()` type introspection
- `ffi.metatype()` metatable association
- `ffi.gc()` finalizer registration
- `ffi.addressof()` address queries
- `ffi.istype()` type checking
- `ffi.tonumber()` conversions
- `ffi.string()` string operations
- `ffi.copy()` memory operations
- `ffi.fill()` memory filling
- `ffi.errno()` errno handling
- Error handling tests
- API completeness checks

## Running Tests

Run all tests:

```bash
cargo test
```

Run specific test file:

```bash
cargo test --test ctype_test
cargo test --test functional_test
cargo test --test parser_test
cargo test --test integration_test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run tests in verbose mode:

```bash
cargo test -- --nocapture --test-threads=1
```
