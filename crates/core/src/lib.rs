//! nya-core: Minimal core library for the nya programming language
//!
//! This crate contains the fundamental types and traits that are always available
//! in every nya program. It's the equivalent of Rust's `core` crate.

#![no_std]
#![allow(incomplete_features)]

extern crate alloc;

/// The nya prelude - automatically imported in every module
pub mod prelude {
    pub use crate::option::{Option, Option::*};
    pub use crate::result::{Result, Result::*};
    pub use crate::iter::{Iterator, IntoIterator};
    pub use crate::string::String;
    pub use crate::vec::Vec;
    pub use crate::boxed::Box;
    pub use crate::rc::Rc;
    pub use crate::arc::Arc;
    pub use crate::cell::{Cell, RefCell};
    pub use crate::fmt::{Debug, Display, Formatter};
    pub use crate::format;
    pub use crate::print;
    pub use crate::println;
    pub use crate::eprint;
    pub use crate::eprintln;
    pub use crate::mem::{size_of, align_of, transmute, replace, swap, take, forget, ManuallyDrop};
    pub use crate::ops::{Deref, DerefMut, Fn, FnMut, FnOnce, Add, Sub, Mul, Div, Rem, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, Index, IndexMut, Range, RangeFrom, RangeTo, RangeFull, RangeInclusive, RangeToInclusive};
    pub use crate::marker::{Copy, Send, Sync, Sized, Unpin, PhantomData};
    pub use crate::ptr::{NonNull, null, null_mut};
    pub use crate::slice::Iter;
    pub use crate::str::Str;
    pub use crate::num::{Int, Float, Signed, Unsigned};
    pub use crate::convert::{From, Into, TryFrom, TryInto, AsRef, AsMut};
    pub use crate::borrow::{Borrow, BorrowMut, ToOwned};
    pub use crate::hash::{Hash, Hasher};
    pub use crate::default::Default;
    pub use crate::clone::Clone;
    pub use crate::drop::Drop;
    pub use crate::panic::{panic, assert, assert_eq, assert_ne, unreachable, unreachable_unchecked, todo, unimplemented};
    
    // Primitive types
    pub use crate::primitives::{bool, char, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64};
}

/// Primitive type definitions (these map to Rust primitives)
pub mod primitives {
    pub use core::primitive::*;
}

/// Option type - represents optional values
pub mod option {
    use core::fmt;
    
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Option<T> {
        None,
        Some(T),
    }
    
    impl<T> Option<T> {
        pub const fn is_some(&self) -> bool {
            matches!(self, Option::Some(_))
        }
        
        pub const fn is_none(&self) -> bool {
            matches!(self, Option::None)
        }
        
        pub fn unwrap(self) -> T {
            match self {
                Option::Some(v) => v,
                Option::None => panic!("called `Option::unwrap()` on a `None` value"),
            }
        }
        
        pub fn expect(self, msg: &str) -> T {
            match self {
                Option::Some(v) => v,
                Option::None => panic!("{}", msg),
            }
        }
        
        pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U> {
            match self {
                Option::Some(v) => Option::Some(f(v)),
                Option::None => Option::None,
            }
        }
        
        pub fn and_then<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> Option<U> {
            match self {
                Option::Some(v) => f(v),
                Option::None => Option::None,
            }
        }
    }
    
    impl<T: fmt::Debug> fmt::Debug for Option<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Option::Some(v) => write!(f, "Some({:?})", v),
                Option::None => write!(f, "None"),
            }
        }
    }
}

/// Result type - represents success or failure
pub mod result {
    use core::fmt;
    
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Result<T, E> {
        Ok(T),
        Err(E),
    }
    
    impl<T, E> Result<T, E> {
        pub const fn is_ok(&self) -> bool {
            matches!(self, Result::Ok(_))
        }
        
        pub const fn is_err(&self) -> bool {
            matches!(self, Result::Err(_))
        }
        
        pub fn unwrap(self) -> T
        where
            E: fmt::Debug,
        {
            match self {
                Result::Ok(v) => v,
                Result::Err(e) => panic!("called `Result::unwrap()` on an `Err` value: {:?}", e),
            }
        }
        
        pub fn expect(self, msg: &str) -> T
        where
            E: fmt::Debug,
        {
            match self {
                Result::Ok(v) => v,
                Result::Err(_) => panic!("{}", msg),
            }
        }
        
        pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result<U, E> {
            match self {
                Result::Ok(v) => Result::Ok(f(v)),
                Result::Err(e) => Result::Err(e),
            }
        }
        
        pub fn map_err<F, O: FnOnce(E) -> F>(self, f: O) -> Result<T, F> {
            match self {
                Result::Ok(v) => Result::Ok(v),
                Result::Err(e) => Result::Err(f(e)),
            }
        }
        
        pub fn and_then<U, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<U, E> {
            match self {
                Result::Ok(v) => f(v),
                Result::Err(e) => Result::Err(e),
            }
        }
        
        pub fn or_else<F, O: FnOnce(E) -> Result<T, F>>(self, f: O) -> Result<T, F> {
            match self {
                Result::Ok(v) => Result::Ok(v),
                Result::Err(e) => f(e),
            }
        }
    }
    
    impl<T: fmt::Debug, E: fmt::Debug> fmt::Debug for Result<T, E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Result::Ok(v) => write!(f, "Ok({:?})", v),
                Result::Err(e) => write!(f, "Err({:?})", e),
            }
        }
    }
}

/// Iterator trait
pub mod iter {
    pub use core::iter::{Iterator, IntoIterator, FromIterator};
}

/// String type
pub mod string {
    use alloc::string::String as StdString;
    use core::fmt;
    use core::ops::Deref;
    
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct String(StdString);
    
    impl String {
        pub fn new() -> Self {
            String(StdString::new())
        }
        
        pub fn from_str(s: &str) -> Self {
            String(StdString::from(s))
        }
        
        pub fn push_str(&mut self, s: &str) {
            self.0.push_str(s)
        }
        
        pub fn as_str(&self) -> &str {
            &self.0
        }
        
        pub fn len(&self) -> usize {
            self.0.len()
        }
        
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }
    
    impl Deref for String {
        type Target = str;
        
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    
    impl fmt::Debug for String {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }
    
    impl fmt::Display for String {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(&self.0, f)
        }
    }
    
    impl From<&str> for String {
        fn from(s: &str) -> Self {
            String::from_str(s)
        }
    }
    
    impl From<StdString> for String {
        fn from(s: StdString) -> Self {
            String(s)
        }
    }
}

/// Vec type
pub mod vec {
    use alloc::vec::Vec as StdVec;
    use core::fmt;
    use core::ops::{Deref, DerefMut, Index, IndexMut};
    
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Vec<T>(StdVec<T>);
    
    impl<T> Vec<T> {
        pub fn new() -> Self {
            Vec(StdVec::new())
        }
        
        pub fn with_capacity(cap: usize) -> Self {
            Vec(StdVec::with_capacity(cap))
        }
        
        pub fn push(&mut self, value: T) {
            self.0.push(value)
        }
        
        pub fn pop(&mut self) -> Option<T> {
            self.0.pop()
        }
        
        pub fn len(&self) -> usize {
            self.0.len()
        }
        
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
        
        pub fn clear(&mut self) {
            self.0.clear()
        }
        
        pub fn capacity(&self) -> usize {
            self.0.capacity()
        }
        
        pub fn reserve(&mut self, additional: usize) {
            self.0.reserve(additional)
        }
    }
    
    impl<T> Default for Vec<T> {
        fn default() -> Self {
            Vec::new()
        }
    }
    
    impl<T> Deref for Vec<T> {
        type Target = [T];
        
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    
    impl<T> DerefMut for Vec<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    
    impl<T> Index<usize> for Vec<T> {
        type Output = T;
        
        fn index(&self, index: usize) -> &Self::Output {
            &self.0[index]
        }
    }
    
    impl<T> IndexMut<usize> for Vec<T> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            &mut self.0[index]
        }
    }
    
    impl<T: fmt::Debug> fmt::Debug for Vec<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }
    
    impl<T> IntoIterator for Vec<T> {
        type Item = T;
        type IntoIter = alloc::vec::IntoIter<T>;
        
        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
    
    impl<'a, T> IntoIterator for &'a Vec<T> {
        type Item = &'a T;
        type IntoIter = core::slice::Iter<'a, T>;
        
        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }
    
    impl<'a, T> IntoIterator for &'a mut Vec<T> {
        type Item = &'a mut T;
        type IntoIter = core::slice::IterMut<'a, T>;
        
        fn into_iter(self) -> Self::IntoIter {
            self.0.iter_mut()
        }
    }
}

/// Box type for heap allocation
pub mod boxed {
    use alloc::boxed::Box as StdBox;
    use core::fmt;
    use core::ops::{Deref, DerefMut};
    
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Box<T: ?Sized>(StdBox<T>);
    
    impl<T> Box<T> {
        pub fn new(value: T) -> Self {
            Box(StdBox::new(value))
        }
        
        pub fn into_inner(self) -> T {
            let boxed = self.0;
            let ptr = StdBox::into_raw(boxed);
            unsafe { *StdBox::from_raw(ptr) }
        }
    }
    
    impl<T: ?Sized> Deref for Box<T> {
        type Target = T;
        
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    
    impl<T: ?Sized> DerefMut for Box<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    
    impl<T: fmt::Debug + ?Sized> fmt::Debug for Box<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }
}

/// Rc - Reference counted pointer
pub mod rc {
    use alloc::rc::Rc as StdRc;
    use core::fmt;
    use core::ops::Deref;
    
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Rc<T: ?Sized>(StdRc<T>);
    
    impl<T> Rc<T> {
        pub fn new(value: T) -> Self {
            Rc(StdRc::new(value))
        }
        
        pub fn strong_count(this: &Self) -> usize {
            StdRc::strong_count(&this.0)
        }
        
        pub fn downgrade(this: &Self) -> Weak<T> {
            Weak(StdRc::downgrade(&this.0))
        }
    }
    
    impl<T: ?Sized> Clone for Rc<T> {
        fn clone(&self) -> Self {
            Rc(StdRc::clone(&self.0))
        }
    }
    
    impl<T: ?Sized> Deref for Rc<T> {
        type Target = T;
        
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    
    impl<T: fmt::Debug + ?Sized> fmt::Debug for Rc<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }
    
    pub struct Weak<T: ?Sized>(alloc::rc::Weak<T>);
    
    impl<T> Weak<T> {
        pub fn upgrade(&self) -> Option<Rc<T>> {
            self.0.upgrade().map(Rc)
        }
    }
}

/// Arc - Atomic reference counted pointer
pub mod arc {
    use alloc::sync::Arc as StdArc;
    use core::fmt;
    use core::ops::Deref;
    
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Arc<T: ?Sized>(StdArc<T>);
    
    impl<T> Arc<T> {
        pub fn new(value: T) -> Self {
            Arc(StdArc::new(value))
        }
        
        pub fn strong_count(this: &Self) -> usize {
            StdArc::strong_count(&this.0)
        }
        
        pub fn downgrade(this: &Self) -> Weak<T> {
            Weak(StdArc::downgrade(&this.0))
        }
    }
    
    impl<T: ?Sized> Clone for Arc<T> {
        fn clone(&self) -> Self {
            Arc(StdArc::clone(&self.0))
        }
    }
    
    impl<T: ?Sized> Deref for Arc<T> {
        type Target = T;
        
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    
    impl<T: fmt::Debug + ?Sized> fmt::Debug for Arc<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }
    
    pub struct Weak<T: ?Sized>(alloc::sync::Weak<T>);
    
    impl<T> Weak<T> {
        pub fn upgrade(&self) -> Option<Arc<T>> {
            self.0.upgrade().map(Arc)
        }
    }
}

/// Cell and RefCell for interior mutability
pub mod cell {
    use core::cell::{Cell as StdCell, RefCell as StdRefCell, Ref, RefMut};
    use core::fmt;
    
    pub struct Cell<T>(StdCell<T>);
    
    impl<T> Cell<T> {
        pub fn new(value: T) -> Self {
            Cell(StdCell::new(value))
        }
        
        pub fn get(&self) -> T
        where
            T: Copy,
        {
            self.0.get()
        }
        
        pub fn set(&self, value: T) {
            self.0.set(value)
        }
        
        pub fn replace(&self, value: T) -> T {
            self.0.replace(value)
        }
        
        pub fn into_inner(self) -> T {
            self.0.into_inner()
        }
    }
    
    impl<T: Copy> Clone for Cell<T> {
        fn clone(&self) -> Self {
            Cell(self.0.clone())
        }
    }
    
    impl<T: fmt::Debug + Copy> fmt::Debug for Cell<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }
    
    pub struct RefCell<T>(StdRefCell<T>);
    
    impl<T> RefCell<T> {
        pub fn new(value: T) -> Self {
            RefCell(StdRefCell::new(value))
        }
        
        pub fn borrow(&self) -> Ref<'_, T> {
            self.0.borrow()
        }
        
        pub fn borrow_mut(&self) -> RefMut<'_, T> {
            self.0.borrow_mut()
        }
        
        pub fn try_borrow(&self) -> Result<Ref<'_, T>, core::cell::BorrowError> {
            self.0.try_borrow()
        }
        
        pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, core::cell::BorrowMutError> {
            self.0.try_borrow_mut()
        }
        
        pub fn into_inner(self) -> T {
            self.0.into_inner()
        }
    }
    
    impl<T: fmt::Debug> fmt::Debug for RefCell<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Debug::fmt(&self.0, f)
        }
    }
}

/// Formatting traits
pub mod fmt {
    pub use core::fmt::{Debug, Display, Formatter, Error as FmtError};
    
    #[macro_export]
    macro_rules! format {
        ($($arg:tt)*) => {
            alloc::format!($($arg)*)
        };
    }
    
    #[macro_export]
    macro_rules! print {
        ($($arg:tt)*) => {
            $crate::prelude::print!($($arg)*)
        };
    }
    
    #[macro_export]
    macro_rules! println {
        () => {
            $crate::prelude::println!()
        };
        ($($arg:tt)*) => {
            $crate::prelude::println!($($arg)*)
        };
    }
    
    #[macro_export]
    macro_rules! eprint {
        ($($arg:tt)*) => {
            $crate::prelude::eprint!($($arg)*)
        };
    }
    
    #[macro_export]
    macro_rules! eprintln {
        () => {
            $crate::prelude::eprintln!()
        };
        ($($arg:tt)*) => {
            $crate::prelude::eprintln!($($arg)*)
        };
    }
}

/// Memory operations
pub mod mem {
    pub use core::mem::{size_of, align_of, transmute, replace, swap, take, forget, ManuallyDrop, MaybeUninit};
}

/// Operator traits
pub mod ops {
    pub use core::ops::{
        Deref, DerefMut, Fn, FnMut, FnOnce,
        Add, Sub, Mul, Div, Rem, Neg, Not,
        BitAnd, BitOr, BitXor, Shl, Shr,
        Index, IndexMut,
        Range, RangeFrom, RangeTo, RangeFull, RangeInclusive, RangeToInclusive,
    };
}

/// Marker traits
pub mod marker {
    pub use core::marker::{Copy, Send, Sync, Sized, Unpin, PhantomData};
    pub use core::clone::Clone;
}

/// Pointer types
pub mod ptr {
    pub use core::ptr::{NonNull, null, null_mut, addr_of, addr_of_mut};
}

/// Slice type
pub mod slice {
    pub use core::slice::{Iter, IterMut, Chunks, ChunksMut, Windows, ChunksExact, ChunksExactMut};
}

/// Str type
pub mod str {
    pub use core::str::{from_utf8, from_utf8_unchecked};
    pub type Str = str;
}

/// Numeric traits
pub mod num {
    pub trait Int: Copy + Clone + PartialEq + Eq + PartialOrd + Ord + core::fmt::Debug {}
    pub trait Float: Copy + Clone + PartialEq + PartialOrd + core::fmt::Debug {}
    pub trait Signed: Int {}
    pub trait Unsigned: Int {}
    
    macro_rules! impl_int {
        ($($t:ty),*) => {
            $(impl Int for $t {})*
        };
    }
    
    macro_rules! impl_float {
        ($($t:ty),*) => {
            $(impl Float for $t {})*
        };
    }
    
    macro_rules! impl_signed {
        ($($t:ty),*) => {
            $(impl Signed for $t {})*
        };
    }
    
    macro_rules! impl_unsigned {
        ($($t:ty),*) => {
            $(impl Unsigned for $t {})*
        };
    }
    
    impl_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
    impl_float!(f32, f64);
    impl_signed!(i8, i16, i32, i64, i128, isize);
    impl_unsigned!(u8, u16, u32, u64, u128, usize);
}

/// Conversion traits
pub mod convert {
    pub use core::convert::{From, Into, TryFrom, TryInto, AsRef, AsMut, identity};
}

/// Borrow traits
pub mod borrow {
    pub use core::borrow::{Borrow, BorrowMut};
    pub use alloc::borrow::ToOwned;
}

/// Hash traits
pub mod hash {
    pub use core::hash::{Hash, Hasher, BuildHasher, BuildHasherDefault};
}

/// Default trait
pub mod default {
    pub use core::default::Default;
}

/// Clone trait
pub mod clone {
    pub use core::clone::Clone;
}

/// Drop trait
pub mod drop {
    pub use core::ops::Drop;
}

/// Panic handling
pub mod panic {
    #[cold]
    #[track_caller]
    pub fn panic(fmt: core::fmt::Arguments<'_>) -> ! {
        core::panic!("{}", fmt)
    }
    
    #[track_caller]
    pub fn assert(cond: bool) {
        if !cond {
            panic!("assertion failed");
        }
    }
    
    #[track_caller]
    pub fn assert_eq<T: core::fmt::Debug + PartialEq>(left: T, right: T) {
        if left != right {
            panic!("assertion failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`", left, right);
        }
    }
    
    #[track_caller]
    pub fn assert_ne<T: core::fmt::Debug + PartialEq>(left: T, right: T) {
        if left == right {
            panic!("assertion failed: `(left != right)`\n  left: `{:?}`,\n right: `{:?}`", left, right);
        }
    }
    
    #[track_caller]
    pub fn unreachable() -> ! {
        panic!("internal error: entered unreachable code");
    }
    
    #[track_caller]
    pub fn unreachable_unchecked() -> ! {
        unsafe { core::hint::unreachable_unchecked() }
    }
    
    #[track_caller]
    pub fn todo() -> ! {
        panic!("not yet implemented");
    }
    
    #[track_caller]
    pub fn unimplemented() -> ! {
        panic!("not yet implemented");
    }
}