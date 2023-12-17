// important! this must be sealed to prevent downstream crates from implementing it.
mod private {
    // Theorically, supertraits are not required, but it helps to maintain the list of supported types.
    // This is a trait implemented for types that can be used without dependencies.
    // Debug is required to format step arguments.
    // Clone is required to ensure the semantics of shared arguments are replicated.
    // Serialize is required to send arguments to external runners.
    pub trait SealedIndependentType: std::fmt::Debug + Clone + serde::Serialize {}
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

macro_rules! local_tuple {
    ($($ty:ident),*) => {
        impl<$($ty: SealedIndependentType),*> SealedIndependentType for ($($ty,)*) {}
    };
}

macro_rules! local_array {
    ($($num:tt),*) => {
        $(
            impl<T: SealedIndependentType> SealedIndependentType for [T; $num] {}
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
    std::time::SystemTime,
    std::path::PathBuf
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
local!(T; &[T], Vec<T>, Option<T>, Box<T>, std::marker::PhantomData<T>, std::rc::Rc<T>, std::sync::Arc<T>, std::cell::RefCell<T>);

local!(T; std::ops::Range<T>, std::ops::RangeFrom<T>, std::ops::RangeTo<T>, std::ops::RangeInclusive<T>, std::ops::Bound<T>);

local!(T; std::collections::BTreeSet<T>, std::collections::LinkedList<T>, std::collections::VecDeque<T>, std::collections::BinaryHeap<T>);

impl<K, V> SealedIndependentType for std::collections::HashMap<K, V>
where
    K: SealedIndependentType + PartialEq + Eq + std::hash::Hash,
    V: SealedIndependentType + PartialEq,
{
}
impl<T> SealedIndependentType for std::collections::HashSet<T> where
    T: SealedIndependentType + PartialEq + Eq + std::hash::Hash
{
}

local!(T, U; Result<T, U>, std::collections::BTreeMap<T, U>);

local_a!(T; &'a T, std::borrow::Cow<'a,T>);

local_tuple!(A);
local_tuple!(A, B);
local_tuple!(A, B, C);
local_tuple!(A, B, C, D);

local_tuple!(A, B, C, D, E);
local_tuple!(A, B, C, D, E, F);
local_tuple!(A, B, C, D, E, F, G);
local_tuple!(A, B, C, D, E, F, G, H);

local_tuple!(A, B, C, D, E, F, G, H, I);
local_tuple!(A, B, C, D, E, F, G, H, I, J);
local_tuple!(A, B, C, D, E, F, G, H, I, J, K);
local_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

local_array!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32
);
