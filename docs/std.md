# nya Standard Library

The nya standard library (`std`) provides essential functionality for systems programming, built on top of the minimal `core` library.

## Module Organization

```
std
├── prelude          # Commonly used items (auto-imported)
├── collections      # Data structures
├── io               # Input/Output
├── fs               # Filesystem
├── net              # Networking
├── thread           # Threading
├── time             # Time & Duration
├── process          # Process management
├── env              # Environment variables
├── path             # Path manipulation
├── ffi              # Foreign Function Interface
├── sync             # Synchronization primitives
├── hash             # Hashing
└── fmt              # Formatting
```

---

## Prelude

The following items are automatically imported in every module:

```nya
// Types
Option<T>, Result<T, E>
Iterator, IntoIterator
String, Vec<T>
Box<T>, Rc<T>, Arc<T>
Cell<T>, RefCell<T>

// Traits
Debug, Display, Clone, Copy
Default, PartialEq, Eq, Hash
From, Into, TryFrom, TryInto
AsRef, AsMut, Borrow, BorrowMut
Drop, Fn, FnMut, FnOnce

// Macros
format!, print!, println!, eprint!, eprintln!
assert!, assert_eq!, assert_ne!
```

---

## Collections

### Vec<T> - Dynamic Array

```nya
use std::collections::Vec;

// Creation
var v = Vec::new()
var v = vec![1, 2, 3]
var v = Vec::with_capacity(100)

// Modification
v.push(4)
v.pop()
v.insert(0, 0)
v.remove(0)
v.clear()

// Access
v[0]           // panics if out of bounds
v.get(0)       // Option<&T>
v.get_mut(0)   // Option<&mut T>
v.first()      // Option<&T>
v.last()        // Option<&T>

// Iteration
for item in &v { ... }
for item in &mut v { ... }
for item in v { ... }  // consumes

// Utility
v.len()
v.is_empty()
v.capacity()
v.shrink_to_fit()
v.dedup()
v.sort()
v.reverse()
```

### String - UTF-8 String

```nya
use std::collections::String;

// Creation
var s = String::new()
var s = "hello".to_string()
var s = String::from("hello")
var s = format!("{} {}", "hello", "world")

// Modification
s.push('!')
s.push_str(" world")
s.insert(5, ',')
s.remove(5)
s.clear()

// Access
s.len()           // byte length
s.chars().count() // character count
s.is_empty()
s.chars()         // iterator over chars
s.bytes()         // iterator over bytes
s.get(0..5)       // Option<&str>

// Search
s.contains("hello")
s.starts_with("he")
s.ends_with("lo")
s.find("el")      // Option<usize>
s.rfind("l")

// Transformation
s.to_uppercase()
s.to_lowercase()
s.trim()
s.trim_start()
s.trim_end()
s.replace("old", "new")
s.replacen("a", "b", 1)
s.split(',')      // iterator over &str
s.lines()         // iterator over lines

// Conversion
s.as_str()        // &str
s.as_bytes()      // &[u8]
s.into_bytes()    // Vec<u8>
```

### HashMap<K, V> - Hash Table

```nya
use std::collections::HashMap;

// Creation
var map = HashMap::new()
var map = HashMap::with_capacity(100)
var map = hashmap!{ "key" => "value" }

// Insertion
map.insert("key", "value")
map.entry("key").or_insert("default")
map.entry("key").or_insert_with(|| compute_default())

// Access
map.get("key")              // Option<&V>
map.get_mut("key")          // Option<&mut V>
map["key"]                  // panics if missing
map.get("key").copied()     // Option<V>

// Removal
map.remove("key")
map.clear()

// Iteration
for (k, v) in &map { ... }
for (k, v) in &mut map { ... }
for (k, v) in map { ... }  // consumes

// Utility
map.len()
map.is_empty()
map.capacity()
map.contains_key("key")
map.keys()      // iterator over keys
map.values()    // iterator over values
map.values_mut() // iterator over &mut V
```

### HashSet<T> - Unique Values

```nya
use std::collections::HashSet;

var set = HashSet::new()
set.insert(1)
set.contains(&1)
set.remove(&1)
set.len()
set.is_empty()

// Set operations
let a = hashset!{1, 2, 3}
let b = hashset!{2, 3, 4}

a.union(&b)          // {1, 2, 3, 4}
a.intersection(&b)   // {2, 3}
a.difference(&b)     // {1}
a.symmetric_difference(&b) // {1, 4}
a.is_subset(&b)
a.is_superset(&b)
a.is_disjoint(&b)
```

### BTreeMap / BTreeSet - Ordered Collections

```nya
use std::collections::{BTreeMap, BTreeSet};

// Same API as HashMap/HashSet but maintains sorted order
var map = BTreeMap::new()
map.insert(3, "c")
map.insert(1, "a")
map.insert(2, "b")

for (k, v) in &map {
    println!("{} -> {}", k, v)  // prints in order: 1, 2, 3
}

// Range queries
for (k, v) in map.range(1..3) { ... }
map.first_key_value()   // Option<(&K, &V)>
map.last_key_value()    // Option<(&K, &V)>
map.pop_first()         // Option<(K, V)>
map.pop_last()          // Option<(K, V)>
```

### VecDeque<T> - Double-Ended Queue

```nya
use std::collections::VecDeque;

var dq = VecDeque::new()
dq.push_front(1)
dq.push_back(2)
dq.pop_front()   // Option<T>
dq.pop_back()    // Option<T>
dq.front()       // Option<&T>
dq.back()        // Option<&T>
```

### LinkedList<T> - Doubly Linked List

```nya
use std::collections::LinkedList;

var list = LinkedList::new()
list.push_front(1)
list.push_back(2)
list.pop_front()
list.pop_back()
list.front()
list.back()
```

---

## I/O

### Read Trait

```nya
use std::io::{Read, BufReader};

fn read_all<R: Read>(mut reader: R) -> Result<String, IoError> {
    var mut buf = String::new()
    reader.read_to_string(&mut buf)?;
    Ok(buf)
}

// Buffered reading
let file = File::open("file.txt")?
var reader = BufReader::new(file)
for line in reader.lines() {
    let line = line?
    println!("{}", line)
}
```

### Write Trait

```nya
use std::io::{Write, BufWriter};

fn write_all<W: Write>(mut writer: W, data: &str) -> Result<(), IoError> {
    writer.write_all(data.as_bytes())?
    writer.flush()?
    Ok(())
}

// Buffered writing
let file = File::create("output.txt")?
var writer = BufWriter::new(file)
writer.write_all(b"Hello")?
writer.flush()?
```

### Standard Streams

```nya
use std::io::{stdin, stdout, stderr};

fn read_input() -> String {
    var mut buf = String::new()
    stdin().read_line(&mut buf).unwrap()
    buf.trim().to_string()
}

fn write_output() {
    stdout().write_all(b"Hello").unwrap()
    stdout().flush().unwrap()
    
    eprint!("Error: ")
    eprintln!("Something went wrong")
}
```

### Files

```nya
use std::fs::File;
use std::io::{Read, Write};

fn read_file(path: &str) -> Result<String, IoError> {
    var mut file = File::open(path)?
    var mut contents = String::new()
    file.read_to_string(&mut contents)?
    Ok(contents)
}

fn write_file(path: &str, data: &str) -> Result<(), IoError> {
    var mut file = File::create(path)?
    file.write_all(data.as_bytes())?
    file.flush()?
    Ok(())
}
```

---

## Filesystem

```nya
use std::fs::{File, read_dir, create_dir_all, remove_file, remove_dir_all, metadata, Metadata};

// Directory operations
create_dir_all("path/to/dir")?
read_dir(".")?  // Iterator over DirEntry

for entry in read_dir(".")? {
    let entry = entry?
    println!("{}", entry.path().display())
    println!("{}", entry.file_name().to_string_lossy())
    println!("is_dir: {}", entry.metadata()?.is_dir())
}

// File operations
remove_file("file.txt")?
remove_dir_all("dir")?

let meta = metadata("file.txt")?
meta.is_file()
meta.is_dir()
meta.len()           // file size in bytes
meta.modified()?     // SystemTime
meta.created()?      // SystemTime
```

### DirEntry

```nya
struct DirEntry {
    path: PathBuf,
    file_name: OsString,
    metadata: Metadata,
}
```

---

## Networking

```nya
use std::net::{TcpStream, TcpListener, UdpSocket, ToSocketAddrs, SocketAddr};

// TCP Client
fn tcp_client() -> Result<(), IoError> {
    var stream = TcpStream::connect("127.0.0.1:8080")?
    stream.write_all(b"Hello")?
    stream.flush()?
    
    var mut buf = [0u8; 1024]
    let n = stream.read(&mut buf)?
    println!("Received: {}", String::from_utf8_lossy(&buf[..n]))
    Ok(())
}

// TCP Server
fn tcp_server() -> Result<(), IoError> {
    var listener = TcpListener::bind("127.0.0.1:8080")?
    
    for stream in listener.incoming() {
        var stream = stream?
        // Handle connection in thread
        std::thread::spawn(move || {
            handle_connection(stream)
        })
    }
    Ok(())
}

// UDP
fn udp_example() -> Result<(), IoError> {
    var socket = UdpSocket::bind("127.0.0.1:0")?
    socket.send_to(b"Hello", "127.0.0.1:8080")?
    
    var mut buf = [0u8; 1024]
    let (n, src) = socket.recv_from(&mut buf)?
    println!("Received {} bytes from {}", n, src)
    Ok(())
}
```

---

## Threading

```nya
use std::thread::{spawn, JoinHandle, current, sleep, park, Thread, ThreadId};
use std::time::Duration;

// Spawning threads
fn thread_example() {
    var handle = spawn(|| {
        println!("Hello from thread!")
    })
    handle.join().unwrap()
}

// Thread with return value
fn thread_with_return() -> Result<(), Error> {
    var handle = spawn(|| {
        expensive_computation()
    })
    let result = handle.join()?
    Ok(())
}

// Thread info
fn thread_info() {
    let thread = current()
    println!("Thread: {:?}", thread.id())
    println!("Thread name: {:?}", thread.name())
}

// Sleep
sleep(Duration::from_secs(1))
sleep(Duration::from_millis(100))

// Parking (thread synchronization)
fn park_example() {
    var thread = spawn(|| {
        println!("Thread parked")
        park()
        println!("Thread unparked")
    })
    
    sleep(Duration::from_millis(100))
    thread.thread().unpark()
    thread.join().unwrap()
}
```

---

## Time

```nya
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

// Duration
let dur = Duration::from_secs(1)
let dur = Duration::from_millis(100)
let dur = Duration::from_nanos(1_000_000_000)

dur.as_secs()
dur.as_millis()
dur.as_nanos()
dur.as_secs_f64()

// Instant (monotonic clock)
let start = Instant::now()
// ... do work ...
let elapsed = start.elapsed()
println!("Elapsed: {:?}", elapsed)

// SystemTime (wall clock)
let now = SystemTime::now()
let since_epoch = now.duration_since(UNIX_EPOCH)?
let since_epoch = now.elapsed()?

// Time calculations
let future = now + Duration::from_secs(60)
let past = now - Duration::from_secs(60)
```

---

## Process Management

```nya
use std::process::{Command, Child, ExitStatus, Output, Stdio, id, exit};

// Simple command
fn simple_command() {
    let output = Command::new("echo")
        .arg("hello")
        .output()
        .expect("Failed to execute")
    
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout))
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr))
    println!("status: {}", output.status)
}

// With pipes
fn piped_command() -> Result<(), IoError> {
    var mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
    
    child.stdin.as_mut().unwrap().write_all(b"hello")?
    let output = child.wait_with_output()?
    println!("{}", String::from_utf8_lossy(&output.stdout))
    Ok(())
}

// Process info
fn process_info() {
    println!("PID: {}", id())
    // exit(0)  // exit with code
}
```

---

## Environment Variables

```nya
use std::env::{args, var, var_os, set_var, remove_var, vars, current_dir, set_current_dir};

// Arguments
fn print_args() {
    for arg in args() {
        println!("{}", arg)
    }
}

// Environment variables
fn env_example() {
    set_var("MY_VAR", "value")
    println!("{}", var("MY_VAR").unwrap_or_default())
    
    if let Ok(val) = var("PATH") {
        println!("PATH: {}", val)
    }
    
    remove_var("MY_VAR")
    
    for (key, val) in vars() {
        println!("{}={}", key, val)
    }
}

// Working directory
fn dir_example() {
    let cwd = current_dir().unwrap()
    println!("CWD: {}", cwd.display())
    
    set_current_dir("/tmp").unwrap()
}
```

---

## Path Manipulation

```nya
use std::path::{Path, PathBuf, Component, Components, Iter};

fn path_example() {
    let path = Path::new("/home/user/file.txt")
    
    // Components
    println!("file_name: {:?}", path.file_name())        // Some("file.txt")
    println!("extension: {:?}", path.extension())        // Some("txt")
    println!("parent: {:?}", path.parent())              // Some("/home/user")
    println!("ancestors: {:?}", path.ancestors().collect::<Vec<_>>())
    
    // Iteration
    for component in path.components() {
        println!("{:?}", component)
    }
    
    // Normalization
    let path = Path::new("/home/../user/./file.txt")
    println!("canonical: {:?}", path.canonicalize().unwrap())
}

// PathBuf (owned, mutable)
fn pathbuf_example() {
    var mut path = PathBuf::new()
    path.push("home")
    path.push("user")
    path.push("file.txt")
    path.set_extension("rs")
    println!("{}", path.display())  // /home/user/file.rs
    
    path.pop()  // removes file.rs
    println!("{}", path.display())  // /home/user
}
```

---

## FFI (Foreign Function Interface)

```nya
use std::ffi::{CStr, CString, OsStr, OsString};
use std::os::raw::{c_char, c_int, c_void};

// C string conversion
fn c_string_example() {
    let c_str = CString::new("Hello").unwrap()
    let ptr: *const c_char = c_str.as_ptr()
    
    // Back to Rust
    unsafe {
        let c_str = CStr::from_ptr(ptr)
        let rust_str = c_str.to_str().unwrap()
        println!("{}", rust_str)
    }
}

// OS string (platform-native)
fn os_string_example() {
    let os_str = OsStr::new("path/to/file")
    let os_string = OsString::from("path/to/file")
    
    // Platform-specific
    #[cfg(unix)]
    use std::os::unix::ffi::OsStrExt;
    let bytes = os_str.as_bytes()
    
    #[cfg(windows)]
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = os_str.encode_wide().collect()
}

// Raw pointers
unsafe fn raw_pointer_example() {
    let x = 42
    let ptr = &x as *const i32
    let mut_ptr = &mut x as *mut i32
    
    println!("{}", *ptr)
    *mut_ptr = 100
}
```

---

## Synchronization

```nya
use std::sync::{Mutex, RwLock, Condvar, Once, Barrier, Arc, Weak};

// Mutex - mutual exclusion
fn mutex_example() {
    let mutex = Mutex::new(0)
    {
        let mut guard = mutex.lock().unwrap()
        *guard += 1
    }  // guard dropped, mutex unlocked
    
    // With Arc for sharing
    let counter = Arc::new(Mutex::new(0))
    var handles = vec![]
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter)
        let handle = spawn(move || {
            let mut num = counter.lock().unwrap()
            *num += 1
        })
        handles.push(handle)
    }
    
    for handle in handles {
        handle.join().unwrap()
    }
    
    println!("Result: {}", *counter.lock().unwrap())
}

// RwLock - multiple readers, single writer
fn rwlock_example() {
    let lock = RwLock::new(0)
    
    // Multiple readers
    {
        let r1 = lock.read().unwrap()
        let r2 = lock.read().unwrap()
        println!("{} {}", *r1, *r2)
    }
    
    // Single writer
    {
        let mut w = lock.write().unwrap()
        *w += 1
    }
}

// Condvar - condition variable
fn condvar_example() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()))
    let (lock, cvar) = &*pair
    
    let pair2 = Arc::clone(&pair)
    let handle = spawn(move || {
        let (lock, cvar) = &*pair2
        let mut started = lock.lock().unwrap()
        *started = true
        cvar.notify_one()
    })
    
    let mut started = lock.lock().unwrap()
    while !*started {
        started = cvar.wait(started).unwrap()
    }
    println!("Notified!")
    handle.join().unwrap()
}

// Once - one-time initialization
static INIT: Once = Once::new()

fn initialize() {
    INIT.call_once(|| {
        println!("Initializing...")
    })
}

// Barrier - synchronization point
fn barrier_example() {
    let barrier = Arc::new(Barrier::new(3))
    var handles = vec![]
    
    for _ in 0..3 {
        let b = Arc::clone(&barrier)
        let handle = spawn(move || {
            println!("Waiting at barrier...")
            b.wait()
            println!("Past barrier!")
        })
        handles.push(handle)
    }
    
    for handle in handles {
        handle.join().unwrap()
    }
}

// Arc - atomic reference counting
fn arc_example() {
    let data = Arc::new(100)
    var handles = vec![]
    
    for _ in 0..5 {
        let data = Arc::clone(&data)
        let handle = spawn(move || {
            println!("{}", *data)
        })
        handles.push(handle)
    }
    
    for handle in handles {
        handle.join().unwrap()
    }
}

// Weak - weak reference (breaks cycles)
fn weak_example() {
    let strong = Arc::new(100)
    let weak = Arc::downgrade(&strong)
    
    if let Some(strong) = weak.upgrade() {
        println!("{}", *strong)
    }
}
```

---

## Hashing

```nya
use std::hash::{Hash, Hasher, BuildHasher, BuildHasherDefault};
use ahash::RandomState;

// Implementing Hash
#[derive(Hash)]
struct Point {
    x: i32,
    y: i32,
}

// Custom hash
fn custom_hash() {
    let mut hasher = RandomState::new().build_hasher()
    hasher.write(b"hello")
    let hash = hasher.finish()
    println!("{:x}", hash)
}

// HashMap with custom hasher
use std::collections::HashMap;

type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<ahash::AHasher>>;

var map = FastHashMap::new()
map.insert("key", "value")
```

---

## Formatting

```nya
use std::fmt::{Debug, Display, Formatter, format, print, println, eprint, eprintln};

// Debug trait
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

// Display trait
impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn formatting_examples() {
    let p = Point { x: 1, y: 2 }
    
    // Debug
    println!("{:?}", p)      // Point { x: 1, y: 2 }
    println!("{:#?}", p)     // pretty
    
    // Display
    println!("{}", p)        // (1, 2)
    
    // Format macro
    let s = format!("Point: {}", p)
    println!("{}", s)
    
    // Print macros
    print!("Hello")
    println!("World")
    eprint!("Error: ")
    eprintln!("Something went wrong")
}
```

---

## Common Patterns

### Result Chaining

```nya
fn process() -> Result<(), Error> {
    let data = fetch()?.transform()?.validate()?.save()?;
    Ok(())
}

// With context
fn process_with_context() -> Result<(), MyError> {
    let data = fetch().map_err(|e| MyError::Fetch(e))?
    Ok(())
}
```

### Option Handling

```nya
fn option_examples() {
    let opt: Option<i32> = Some(42)
    
    // unwrap / expect
    let val = opt.unwrap()
    let val = opt.expect("Should be Some")
    
    // map / and_then
    let doubled = opt.map(|x| x * 2)
    let squared = opt.and_then(|x| Some(x * x))
    
    // unwrap_or / unwrap_or_else
    let val = opt.unwrap_or(0)
    let val = opt.unwrap_or_else(|| compute_default())
    
    // if let
    if let Some(x) = opt {
        println!("{}", x)
    }
    
    // match
    match opt {
        Some(x) => println!("{}", x),
        None => println!("None"),
    }
}
```

### Iterator Patterns

```nya
fn iterator_examples() {
    let v = vec![1, 2, 3, 4, 5]
    
    // Basic
    for x in &v { println!("{}", x) }
    
    // map/filter
    let doubled: Vec<_> = v.iter().map(|x| x * 2).collect()
    let evens: Vec<_> = v.iter().filter(|x| x % 2 == 0).collect()
    
    // fold/reduce
    let sum = v.iter().fold(0, |acc, x| acc + x)
    let product = v.iter().product::<i32>()
    
    // find
    let first_even = v.iter().find(|x| x % 2 == 0)
    
    // enumerate
    for (i, x) in v.iter().enumerate() {
        println!("{}: {}", i, x)
    }
    
    // zip
    let pairs: Vec<_> = v.iter().zip(v.iter().skip(1)).collect()
}
```

---

*nya Standard Library v0.1.0 - See [Language Guide](language-guide.md) for syntax reference*