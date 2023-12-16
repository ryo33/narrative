// important! this must be sealed to prevent downstream crates from implementing it.
mod private {
    // Theorically, supertraits are not required, but it helps to maintain the list of supported types.
    // This is a trait implemented for types that can be used without dependencies.
    // Debug is required to format step arguments.
    // Clone is required to ensure the semantics of shared arguments are replicated.
    pub trait SealedIndependentType: std::fmt::Debug + Clone {}
}
use private::SealedIndependentType;

pub trait IndependentType: SealedIndependentType {}

impl<T: SealedIndependentType> IndependentType for T {}

macro_rules! local {
    ($($ty:ty),*) => {
        $(
            impl SealedIndependentType for $ty {}
        )*
    };
    ($gen1:tt; $($ty:ty),*) => {
        $(
            impl<$gen1: SealedIndependentType> SealedIndependentType for $ty {}
        )*
    };
    ($gen1:ident, $gen2:ident; $($ty:ty),*) => {
        $(
            impl<$gen1: SealedIndependentType, $gen2: SealedIndependentType> SealedIndependentType for $ty {}
        )*
    };
}
macro_rules! local_a {
    ($gen1:tt; $($ty:ty),*) => {
        $(
            impl<'a, $gen1: SealedIndependentType> SealedIndependentType for $ty {}
        )*
    };
}
local!(
    String,
    &str,
    (),
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64
);
local!(
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize,
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize
);
local!(
    std::time::Duration,
    std::time::Instant,
    std::time::SystemTime
);
local!(std::ffi::OsString, std::ffi::CString);
local!(
    std::net::IpAddr,
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::net::SocketAddr,
    std::net::SocketAddrV4,
    std::net::SocketAddrV6
);
local!(
    std::fs::FileType,
    std::fs::OpenOptions,
    std::fs::Permissions,
    std::path::PathBuf
);
local!(
    std::alloc::Layout,
    std::alloc::LayoutError,
    std::alloc::System,
    std::env::VarError,
    std::io::SeekFrom,
    std::num::FpCategory,
    std::any::TypeId,
    std::cmp::Ordering,
    std::thread::Thread,
    std::thread::ThreadId,
    std::io::ErrorKind
);
local!(T; &[T], Vec<T>, Option<T>, Box<T>, std::pin::Pin<T>, std::rc::Rc<T>, std::sync::Arc<T>, std::cell::RefCell<T>);
local!(T; std::collections::BTreeSet<T>, std::collections::LinkedList<T>, std::collections::VecDeque<T>, std::collections::BinaryHeap<T>);
local!(T; std::ops::Range<T>, std::ops::RangeFrom<T>, std::ops::RangeTo<T>, std::ops::RangeInclusive<T>, std::ops::RangeToInclusive<T>, std::ops::Bound<T>);
local!(T; std::future::Pending<T>, std::future::Ready<T>, std::iter::Empty<T>, std::iter::Once<T>, std::iter::Repeat<T>);
local!(T; std::io::Cursor<T>, std::marker::PhantomData<T>, std::mem::Discriminant<T>, std::ptr::NonNull<T>, std::sync::mpsc::Sender<T>);

local!(T, U; Result<T, U>, std::collections::BTreeMap<T, U>);

local_a!(T; &'a T, std::borrow::Cow<'a,T>);

impl<const N: usize> SealedIndependentType for [u8; N] {}
impl<K, V> SealedIndependentType for std::collections::HashMap<K, V>
where
    K: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash,
    V: std::fmt::Debug + Clone + PartialEq,
{
}
impl<T> SealedIndependentType for std::collections::HashSet<T> where
    T: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash
{
}
