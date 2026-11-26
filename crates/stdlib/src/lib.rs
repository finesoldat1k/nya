//! nya standard library
//!
//! This crate provides the standard library for nya, built on top of core.

extern crate alloc;

/// Collections module
pub mod collections {
    pub use alloc::vec::Vec;
    pub use alloc::string::String;
    pub use hashbrown::{HashMap, HashSet};
    pub use alloc::collections::{BTreeMap, BTreeSet, VecDeque, LinkedList};
}

/// IO module
pub mod io {
    pub use std::io::{Read, Write, BufReader, BufWriter, stdin, stdout, stderr, Error, ErrorKind, Result};
}

/// Filesystem module
pub mod fs {
    pub use std::fs::{File, DirEntry, read_dir, create_dir_all, remove_file, remove_dir_all, metadata, Metadata};
}

/// Network module
pub mod net {
    pub use std::net::{TcpStream, TcpListener, UdpSocket, ToSocketAddrs, SocketAddr};
}

/// Thread module
pub mod thread {
    pub use std::thread::{spawn, JoinHandle, current, sleep, park, Thread, ThreadId};
}

/// Synchronization module
pub mod sync {
    pub use std::sync::{Mutex, RwLock, Condvar, Once, Barrier, Arc, Weak};
}

/// Time module
pub mod time {
    pub use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
}

/// Process module
pub mod process {
    pub use std::process::{Command, Child, ExitStatus, Output, Stdio, id, exit};
}

/// Environment module
pub mod env {
    pub use std::env::{args, var, var_os, set_var, remove_var, vars, current_dir, set_current_dir};
}

/// Path module
pub mod path {
    pub use std::path::{Path, PathBuf, Component, Components, Iter};
}

/// FFI module
pub mod ffi {
    pub use std::ffi::{CStr, CString, OsStr, OsString};
    pub use std::os::raw::{c_char, c_int, c_void};
}

/// Synchronization module (re-export)
pub mod sync_alt {
    pub use std::sync::{Mutex, RwLock, Condvar, Once, Barrier, Arc, Weak};
}

/// Hash module
pub mod hash {
    pub use std::hash::{Hash, Hasher, BuildHasher, BuildHasherDefault};
    pub use ahash::RandomState;
}

/// Format module
pub mod fmt {
    pub use core::fmt::{Debug, Display, Formatter};
}

/// The standard library prelude
pub mod prelude {
    pub use crate::collections::{Vec, String, HashMap, HashSet, BTreeMap, BTreeSet, VecDeque, LinkedList};
    pub use crate::io::{Read, Write, BufReader, BufWriter, stdin, stdout, stderr, Error, ErrorKind, Result};
    pub use crate::fs::{File, DirEntry, read_dir, create_dir_all, remove_file, remove_dir_all, metadata, Metadata};
    pub use crate::net::{TcpStream, TcpListener, UdpSocket, ToSocketAddrs, SocketAddr};
    pub use crate::thread::{spawn, JoinHandle, current, sleep, park, Thread, ThreadId};
    pub use crate::sync::{Mutex, RwLock, Condvar, Once, Barrier, Arc, Weak};
    pub use crate::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
    pub use crate::process::{Command, Child, ExitStatus, Output, Stdio, id, exit};
    pub use crate::env::{args, var, var_os, set_var, remove_var, vars, current_dir, set_current_dir};
    pub use crate::path::{Path, PathBuf, Component, Components, Iter};
    pub use crate::fmt::{Debug, Display, Formatter};
    pub use crate::hash::{Hash, Hasher, BuildHasher, BuildHasherDefault, RandomState};
    
    // Re-export core prelude
    pub use core::prelude::*;
}