# LuaFFI

一个用 Rust 实现的 Lua FFI (Foreign Function Interface) 库，为 Lua 5.3 提供 C 语言类型系统和动态库加载功能。

## 特性

- **完整的 C 类型系统**：支持基础类型、指针、数组、结构体、联合体等
- **动态库加载**：运行时加载和调用本地动态库函数
- **C 声明解析**：解析 C 语言类型声明并自动注册类型
- **内存操作**：提供内存分配、拷贝、填充等底层操作
- **类型安全**：类型检查和转换机制
- **跨平台**：支持 Linux、Windows 等主流平台

## 安装

### 构建要求

- Rust 1.70+
- Cargo
- Lua 5.3（自动通过 vendored 特性包含）

### 从源码构建

```bash
# 克隆仓库
git clone <repository-url>
cd luaffi

# 构建项目
cargo build --release

# 运行测试
cargo test
```

构建完成后，生成的动态库位于 `target/release/libluaffi.so` (Linux) 或 `target/release/luaffi.dll` (Windows)。

## 使用示例

### 基础使用

```lua
local ffi = require("luaffi")

-- 定义 C 类型
ffi.cdef[[
    typedef struct Point {
        int x;
        int y;
    } Point;
    
    int printf(const char *fmt, ...);
]]

-- 创建结构体实例
local point = ffi.new("Point", {x = 10, y = 20})
print(point.x, point.y)  -- 输出: 10  20

-- 调用 C 标准库函数
ffi.C.printf("Hello from FFI!\n")
```

### 加载动态库

```lua
local ffi = require("luaffi")

-- 加载自定义动态库
local mylib = ffi.load("mylib")

-- 定义函数签名
ffi.cdef[[
    int my_function(int a, int b);
]]

-- 调用库函数
local result = mylib.my_function(5, 3)
```

### 内存操作

```lua
local ffi = require("luaffi")

-- 分配内存
local buffer = ffi.new("uint8_t[1024]")

-- 填充内存
ffi.fill(buffer, 1024, 0)

-- 拷贝内存
ffi.copy(buffer, "Hello, FFI!", 11)

-- 获取大小
print(ffi.sizeof("int"))  -- 输出: 4
```

## API 参考

### 核心函数

- `ffi.cdef(code)` - 解析并注册 C 类型声明
- `ffi.load(name)` - 加载动态库
- `ffi.new(ctype, [init])` - 创建 C 数据对象
- `ffi.cast(ctype, value)` - 类型转换
- `ffi.typeof(ctype)` - 获取类型信息
- `ffi.metatype(ctype, metatable)` - 设置类型元表

### 内存操作函数

- `ffi.sizeof(ctype)` - 获取类型大小
- `ffi.offsetof(ctype, field)` - 获取字段偏移
- `ffi.addressof(cdata)` - 获取对象地址
- `ffi.copy(dst, src, len)` - 内存拷贝
- `ffi.fill(dst, len, c)` - 内存填充

### 类型转换函数

- `ffi.istype(ctype, obj)` - 类型检查
- `ffi.tonumber(cdata)` - 转换为数字
- `ffi.string(cdata, [len])` - 转换为字符串

### 其他函数

- `ffi.gc(cdata, finalizer)` - 设置垃圾回收器
- `ffi.errno([newval])` - 获取/设置 errno
- `ffi.C` - C 标准库命名空间
- `ffi.nullptr` - 空指针常量

## 支持的 C 类型

### 基础类型

- 布尔类型：`bool`
- 字符类型：`char`, `signed char`, `unsigned char`
- 整数类型：`short`, `int`, `long`, `long long`（及其 unsigned 变体）
- 浮点类型：`float`, `double`, `long double`
- 空类型：`void`

### 固定宽度整数（stdint.h）

- `int8_t`, `int16_t`, `int32_t`, `int64_t`
- `uint8_t`, `uint16_t`, `uint32_t`, `uint64_t`
- `intptr_t`, `uintptr_t`
- `size_t`, `ssize_t`

### 复合类型

- **指针**：`T*`, `T**` 等
- **数组**：`T[N]`, `T[]`
- **结构体**：`struct { ... }`
- **联合体**：`union { ... }`
- **函数指针**：`int (*)(int, int)` 等
- **typedef**：自定义类型别名

## 项目结构

```text
luaffi/
├── src/
│   ├── lib.rs          # 主模块和 API 导出
│   ├── ctype.rs        # C 类型系统实现
│   ├── cdata.rs        # C 数据对象和动态库封装
│   ├── parser.rs       # C 声明解析器
│   ├── ffi_ops.rs      # FFI 操作实现
│   └── dylib.rs        # 动态库加载
├── tests/
│   ├── ctype_test.rs   # 类型系统测试
│   ├── parser_test.rs  # 解析器测试
│   ├── functional_test.rs  # 功能测试
│   └── integration_test.rs # 集成测试
├── Cargo.toml
└── README.md
```

## 测试

运行所有测试：

```bash
cargo test
```

运行特定测试：

```bash
# 类型系统测试
cargo test --test ctype_test

# 解析器测试
cargo test --test parser_test

# 功能测试
cargo test --test functional_test
```

## 依赖项

- [mlua](https://github.com/mlua-rs/mlua) - Lua 绑定
- [nom](https://github.com/rust-bakery/nom) - 解析器组合子
- [libc](https://github.com/rust-lang/libc) - C 标准库绑定
- [phf](https://github.com/rust-phf/rust-phf) - 编译时哈希表

## 兼容性

- **Lua 版本**：Lua 5.3
- **平台**：Linux, Windows, macOS
- **架构**：x86_64, ARM64

## 许可证

[添加您的许可证信息]

## 贡献

欢迎提交 Issue 和 Pull Request。
