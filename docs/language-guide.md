# nya Language Guide

Complete reference for the nya programming language syntax and semantics.

## Table of Contents

1. [Lexical Structure](#lexical-structure)
2. [Variables & Mutability](#variables--mutability)
3. [Types](#types)
4. [Control Flow](#control-flow)
5. [Functions](#functions)
6. [Structs & Enums](#structs--enums)
7. [Pattern Matching](#pattern-matching)
8. [Error Handling](#error-handling)
9. [Modules & Imports](#modules--imports)
10. [Attributes](#attributes)
11. [Cute Keywords](#cute-keywords)

---

## Lexical Structure

### Keywords

```
fn struct enum impl type const var pub static extern
use mod as return if else while for in loop break continue match let mut ref move where trait
meow nya
```

### Operators

```
+ - * / % ^ & | ~ << >> = == != < > <= >= && || !
+= -= *= /= %= ^= &= |= <<= >>=
. .. ..= :: -> => @ # $ ? !
```

### Literals

```nya
// Integers
42          // i32 (default)
42i8        // i8
42u32       // u32
0x2A        // hex
0o52        // octal
0b101010    // binary
1_000_000   // underscores for readability

// Floats
3.14        // f64 (default)
3.14f32     // f32
1e10        // scientific notation
2.5e-3

// Strings
"hello"           // String
"line1\nline2"    // escape sequences
r"raw\nstring"    // raw string (no escapes)

// Characters
'a'       // char
'\n'      // newline
'\x41'    // hex escape
'\u{1F431}' // unicode escape (cat emoji!)

// Booleans
true false
```

### Comments

```nya
// Line comment

/* Block comment */
/* Nested /* block */ comment */
```

---

## Variables & Mutability

### Immutable (Default)

```nya
const PI = 3.14159
const NAME = "nya"
const FLAG = true
```

### Mutable

```nya
var counter = 0
counter = 1  // OK

var mut name = "original"
name = "changed"  // OK
```

### References

```nya
var x = 42
var ref_x = &x        // immutable reference
var mut ref_mut = &mut x  // mutable reference
```

### Type Annotations

```nya
const PI: f64 = 3.14159
var count: u32 = 0
var name: String = "nya"
```

---

## Types

### Primitives

| Type | Size | Range |
|------|------|-------|
| `bool` | 1 byte | `true` / `false` |
| `i8` | 1 byte | -128 to 127 |
| `i16` | 2 bytes | -32,768 to 32,767 |
| `i32` | 4 bytes | -2B to 2B |
| `i64` | 8 bytes | -9E18 to 9E18 |
| `i128` | 16 bytes | huge |
| `isize` | arch | pointer-sized signed |
| `u8` | 1 byte | 0 to 255 |
| `u16` | 2 bytes | 0 to 65,535 |
| `u32` | 4 bytes | 0 to 4B |
| `u64` | 8 bytes | 0 to 18E18 |
| `u128` | 16 bytes | huge |
| `usize` | arch | pointer-sized unsigned |
| `f32` | 4 bytes | IEEE 754 single |
| `f64` | 8 bytes | IEEE 754 double |
| `char` | 4 bytes | Unicode scalar |
| `str` | fat ptr | string slice |

### Compound Types

```nya
// Tuples
var tuple = (1, "hello", true)  // (i32, &str, bool)
var point = (x: 10, y: 20)      // named tuple

// Arrays (fixed size)
var arr: [i32; 5] = [1, 2, 3, 4, 5]

// Slices (dynamic)
var slice: &[i32] = &arr[1..3]

// References
&i32        // immutable reference
&mut i32    // mutable reference

// Raw pointers (unsafe)
*const i32
*mut i32
```

### Type Aliases

```nya
type UserId = u64
type Result<T> = Result<T, String>
type Callback = fn(i32) -> i32
```

---

## Control Flow

### If Expressions

```nya
// Statement
if x > 0 {
    println!("positive")
} else if x < 0 {
    println!("negative")
} else {
    println!("zero")
}

// Expression (returns value)
var result = if x > 0 { "positive" } else { "negative" };

// Ternary-like (if as expression)
var status = if x > 0 { "ok" } else { "error" };
```

### Loops

```nya
// Infinite loop
loop {
    if done { break; }
    process();
}

// While loop
while condition {
    process();
}

// For loop (iterator)
for item in collection {
    process(item);
}

// Labeled loops
'outer: loop {
    'inner: loop {
        break 'outer;  // breaks outer loop
    }
}
```

### Break & Continue

```nya
loop {
    if condition { break; }
    if skip { continue; }
    process();
}

// With values (break from loop expression)
var result = loop {
    if done { break value; }
    compute();
};
```

---

## Functions

### Basic Syntax

```nya
fn add(a: i32, b: i32) -> i32 {
    a + b  // implicit return (no semicolon)
}

// Explicit return
fn div(a: i32, b: i32) -> i32 {
    if b == 0 { return 0; }
    a / b
}
```

### Parameters

```nya
// By value (move)
fn take_ownership(x: String) { ... }

// Immutable reference
fn borrow(x: &String) { ... }

// Mutable reference
fn modify(x: &mut String) { ... }

// Mutable value
fn mutate(var x: i32) { x += 1; }
```

### Generics

```nya
fn identity<T>(x: T) -> T { x }

fn first<T>(list: &[T]) -> Option<&T> {
    list.first()
}

// With bounds
fn print_debug<T: Debug>(x: T) { println!("{:?}", x) }

// Multiple bounds
fn process<T: Debug + Clone>(x: T) { ... }

// Where clause
fn complex<T, U>(t: T, u: U) -> T
where T: Clone, U: Debug {
    t
}
```

### Variadic / Default Parameters

```nya
// Not directly supported - use Option or Builder pattern
fn greet(name: String, greeting: Option<String>) {
    let g = greeting.unwrap_or("Hello".to_string());
    println!("{}, {}!", g, name);
}
```

---

## Structs & Enums

### Structs

```nya
// Basic struct
struct Cat {
    name: String,
    age: u8,
    lives: u8 = 9,  // default value
}

// Tuple struct
struct Point(i32, i32);
struct Color(u8, u8, u8);

// Unit struct
struct Marker;

// With generics
struct Pair<T, U> {
    first: T,
    second: U,
}

// Visibility
struct PublicCat {
    pub name: String,
    age: u8,  // private by default
}
```

### Struct Methods

```nya
impl Cat {
    // Cute constructor name!
    fn meow(name: String, age: u8) -> Self {
        Self { name, age, lives: 9 }
    }

    // Immutable method
    fn speak(&self) {
        println!("{} says: Nya~!", self.name)
    }

    // Mutable method
    pub fn birthday(&mut self) {
        self.age += 1
        if self.age > 15 { self.lives -= 1 }
    }

    // Static method
    static fn species() -> String {
        "Felis catus".to_string()
    }

    // Generic method
    fn clone_with_age<U: Into<u8>>(&self, new_age: U) -> Self {
        Self { name: self.name.clone(), age: new_age.into(), lives: self.lives }
    }
}
```

### Enums

```nya
// Basic enum
enum Option<T> {
    Some(T),
    None,
}

// With data
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Complex variants
enum Event {
    KeyPress(char),
    MouseMove { x: i32, y: i32 },
    Resize(i32, i32),
    Quit,
}

// Unit variant
enum State {
    Active,
    Inactive,
    Pending,
}
```

### Enum Methods

```nya
impl<T> Option<T> {
    fn is_some(&self) -> bool { matches!(self, Option::Some(_)) }
    fn is_none(&self) -> bool { matches!(self, Option::None) }
    
    fn unwrap(self) -> T {
        match self {
            Option::Some(v) => v,
            Option::None => panic!("unwrap on None"),
        }
    }
    
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
        match self {
            Option::Some(v) => Option::Some(f(v)),
            Option::None => Option::None,
        }
    }
}
```

---

## Pattern Matching

### Match Expressions

```nya
match value {
    1 => println!("one"),
    2 => println!("two"),
    3..=10 => println!("three to ten"),
    _ => println!("other"),
}

// Match with binding
match option {
    Option::Some(x) => println!("Got: {}", x),
    Option::None => println!("Nothing"),
}

// Match with guards
match number {
    n if n % 2 == 0 => println!("even"),
    n if n % 3 == 0 => println!("divisible by 3"),
    _ => println!("other"),
}

// Match on enum
match result {
    Result::Ok(value) => process(value),
    Result::Err(err) => handle_error(err),
}

// Match on struct
match point {
    Point { x: 0, y: 0 } => println!("origin"),
    Point { x, y } => println!("({}, {})", x, y),
}
```

### If Let / While Let

```nya
if let Option::Some(x) = option {
    println!("Got: {}", x);
}

while let Option::Some(item) = iterator.next() {
    process(item);
}
```

---

## Error Handling

### Result Type

```nya
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### The `?` Operator

```nya
fn read_file(path: &str) -> Result<String, IoError> {
    let mut file = File::open(path)?;  // returns Err on failure
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Chaining
fn process() -> Result<(), Error> {
    let data = fetch_data()?.transform()?.save()?;
    Ok(())
}
```

### Custom Errors

```nya
enum MyError {
    Io(IoError),
    Parse(ParseError),
    Custom(String),
}

impl From<IoError> for MyError {
    fn from(err: IoError) -> Self { MyError::Io(err) }
}
```

### Panic

```nya
fn unwrap_or_panic<T>(opt: Option<T>) -> T {
    match opt {
        Some(v) => v,
        None => panic!("Expected Some, got None"),
    }
}

// Assertions
assert!(condition);
assert_eq!(left, right);
assert_ne!(left, right);
```

---

## Modules & Imports

### Module Declaration

```nya
// Inline module
mod utils {
    pub fn helper() { ... }
}

// File module (utils.nya or utils/mod.nya)
mod utils;

// Nested modules
mod net {
    pub mod tcp;
    pub mod udp;
}
```

### Imports

```nya
// Import specific items
use std::io::{Read, Write, stdin, stdout};

// Import with alias
use std::collections::HashMap as Map;

// Import all (use sparingly)
use std::prelude::*;

// Relative imports
use crate::utils::helper;
use super::parent_module::item;
use self::submodule::item;
```

### Visibility

```nya
pub fn public_function() { ... }     // public
fn private_function() { ... }         // private (default)
pub(crate) fn crate_visible() { ... } // crate-visible
```

---

## Attributes

```nya
// Test attribute
#[test]
fn test_something() { ... }

#[test]
#[should_panic]
fn test_panic() { ... }

// Conditional compilation
#[cfg(target_os = "linux")]
fn linux_only() { ... }

// Inline hint
#[inline]
fn fast_function() { ... }

// Deprecated
#[deprecated(since = "0.2", note = "use new_function instead")]
fn old_function() { ... }

// Custom attributes
#[my_custom_attr]
fn annotated() { ... }
```

---

## Cute Keywords

nya has special "cute" keywords that make the language delightful:

| Keyword | Purpose | Example |
|---------|---------|---------|
| `meow` | Constructor | `fn meow() -> Self { ... }` |
| `nya` | Language name / keyword | `const VERSION = "nya 0.1"` |

### Why `meow` for constructors?

```nya
struct Cat {
    name: String,
}

impl Cat {
    fn meow(name: String) -> Self {  // instead of `new`
        Self { name }
    }
}

// Usage
var cat = Cat::meow("Mochi".to_string());
```

### Error Messages

nya's error messages include cute emojis:

```
error: cannot borrow mutably while mutably borrowed
  --> src/main.nya:10:5
   |
10 | var ref = &mut cat;
   |     ^^^^^^ 😿 This kitty has been borrowed!
```

---

## Advanced Topics

### Unsafe Code

```nya
unsafe fn dangerous() {
    let ptr = 0xdeadbeef as *mut i32;
    *ptr = 42;  // dereference raw pointer
}

unsafe {
    dangerous();
}
```

### FFI

```nya
extern "C" {
    fn printf(format: *const c_char, ...) -> c_int;
}

#[no_mangle]
pub extern "C" fn my_callback() {
    println!("Called from C!");
}
```

### Const Generics

```nya
struct Array<T, const N: usize> {
    data: [T; N],
}

fn process<T, const N: usize>(arr: &Array<T, N>) { ... }
```

### Async (Future)

```nya
// Not yet implemented - planned for v0.2
async fn fetch(url: &str) -> Result<Response, Error> { ... }
```

---

## Style Guide

### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Types | PascalCase | `MyStruct`, `MyEnum` |
| Functions | snake_case | `my_function` |
| Variables | snake_case | `my_var` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_SIZE` |
| Modules | snake_case | `my_module` |
| Traits | PascalCase | `Debug`, `Clone` |

### Formatting

```nya
// Use 4 spaces for indentation
fn main() {
    let x = 1;
    if x > 0 {
        println!("positive");
    }
}

// Trailing commas in multiline
struct Cat {
    name: String,
    age: u8,
    lives: u8 = 9,
}

// Where clauses on new line
fn complex<T, U>(t: T, u: U) -> T
where
    T: Clone,
    U: Debug,
{
    t
}
```

---

## Standard Library Overview

### Core Types (always available)

- `Option<T>`, `Result<T, E>`
- `Iterator`, `IntoIterator`
- `String`, `Vec<T>`
- `Box<T>`, `Rc<T>`, `Arc<T>`
- `Cell<T>`, `RefCell<T>`
- `Clone`, `Copy`, `Debug`, `Display`
- `Default`, `PartialEq`, `Eq`
- `Hash`, `From`, `Into`

### Standard Library (`std`)

```nya
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};
use std::io::{Read, Write, BufReader, BufWriter};
use std::fs::{File, read_dir, create_dir_all};
use std::net::{TcpStream, TcpListener};
use std::thread::{spawn, JoinHandle};
use std::sync::{Mutex, RwLock, Arc};
use std::time::{Instant, Duration};
```

---

## Quick Reference

### Common Patterns

```nya
// Builder pattern
struct CatBuilder {
    name: String,
    age: u8,
}

impl CatBuilder {
    fn new(name: String) -> Self { Self { name, age: 0 } }
    fn age(mut self, age: u8) -> Self { self.age = age; self }
    fn build(self) -> Cat { Cat { name: self.name, age: self.age, lives: 9 } }
}

// Usage
var cat = CatBuilder::new("Mochi".to_string()).age(3).build();
```

```nya
// RAII - Resource Acquisition Is Initialization
struct FileHandle {
    file: File,
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        self.file.flush().ok();
    }
}
```

```nya
// Extension traits
trait StringExt {
    fn meowify(&self) -> String;
}

impl StringExt for String {
    fn meowify(&self) -> String {
        format!("{} 🐱", self)
    }
}

// Usage
let s = "hello".to_string().meowify(); // "hello 🐱"
```

---

*nya v0.1.0 - For the latest updates, see [GitHub](https://github.com/nya-lang/nya)*