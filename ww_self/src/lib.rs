use shrink_wrap::prelude::*;
use ww_numeric::{NumericAnyType, NumericBaseType};
use ww_version::{CompactVersion, FullVersion};

pub struct ApiBundle<'i> {
    /// API entry point.
    pub root: ApiLevel<'i>,
    /// Deduplicated array of types collected from all API levels, referred to by [Type::OutOfLine].
    pub types: RefVec<'i, Type<'i>>,
    /// Deduplicated array of all external dependencies, referred to by [TypeDefinitionSource::GlobalFull].
    pub ext_crates: RefVec<'i, FullVersion<'i>>
}

pub struct ApiLevel<'i> {
    pub docs: &'i str,
    pub ident: &'i str,
    // pub source_location?
    pub items: RefVec<'i, ApiItem<'i>>,
}

pub struct ApiItem<'i> {
    pub id: UNib32,
    pub multiplicity: Multiplicity,
    pub ident: &'i str,
    pub docs: &'i str,
    pub kind: ApiItemKind<'i>,
}

pub enum Multiplicity {
    Flat,
    Array, // size bound?
}

pub enum ApiItemKind<'i> {
    Method {
        args: RefVec<'i, Argument<'i>>,
        return_ty: Option<Type<'i>>,
    },
    Property {
        ty: Type<'i>,
        access: PropertyAccess,
    },
    Stream {
        ty: Type<'i>,
        is_up: bool,
    },
    Trait {
        level: RefBox<'i, ApiLevel<'i>>,
    },
    Reserved,
}

pub struct Argument<'i> {
    pub ident: &'i str,
    pub ty: Type<'i>,
}

pub enum PropertyAccess {
    /// Property is not going to change, observe not available
    Const,
    /// Property can only be read, but can change and be observed for changes
    ReadOnly,
    /// Property can be read, written and observed for changes
    ReadWrite,
    /// Property can only be written
    WriteOnly,
}

#[derive_shrink_wrap]
#[ww_repr(unib32)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Type<'i> {
    /// 1-bit, alignment of one-bit, same as `UB(UBits(1))` but serialized with only 1 nibble because bool is used very often.
    Bool,

    NumericBase(NumericBaseType),
    NumericAny(NumericAnyType<'i>),

    // Reserved Type with discriminant length of 1 nibble
    // Reserved6,
    // Reserved Type with discriminant length of 1 nibble
    // Reserved7,
    /// Variable length Unicode string
    String,
    /// Variable length `Vec<T>`
    Vec(RefBox<'i, Type<'i>>),
    /// Fixed size array `[T; len]`
    Array {
        len: u32,
        ty: RefBox<'i, Type<'i>>,
    },
    /// Variable length tuple `(T1, T2, ...)`
    Tuple(RefVec<'i, Type<'i>>),
    /// User defined struct
    Struct(ItemStruct<'i>),
    /// User defined enum
    Enum,
    /// Flag followed by Optional `T` if true and nothing otherwise.
    Option {
        /// If true then flag is popped from the stack; otherwise it is read from the buffer
        is_flag_on_stack: bool,
        some_ty: RefBox<'i, Type<'i>>,
    },
    /// Flag followed by `T` if flag is true and `E` otherwise.
    Result {
        /// If true then flag is popped from the stack; otherwise it is read from the buffer
        is_flag_on_stack: bool,
        ok_ty: RefBox<'i, Type<'i>>,
        err_ty: RefBox<'i, Type<'i>>,
    },
    /// Read bool and put it onto "flag stack".
    /// When serializing: must do the reverse operation for all Options and Results that have is_flag_on_stack set to true.
    Flag,

    /// Type definition from ApiBundle types array.
    OutOfLine {
        idx: u32,
    },
}

pub struct TypeMeta<'i> {
    def: Type<'i>,
    source: TypeDefinitionSource,
}

pub enum TypeDefinitionSource {
    /// Type was defined in the same crate as ApiLevel that refers to itl.
    Local,
    /// Type was defined in an external crate that have a global ID assigned to it.
    GlobalCompact(CompactVersion),
    /// Type was defined in an external crate without global ID. One deduplicated array of names is kept in [ApiBundle].
    GlobalFull {
        /// Index into [ApiBundle] ext_crates array.
        idx: u32
    }
}

#[derive_shrink_wrap]
#[ww_repr(unib32)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Value<'i> {
    Bool(bool),
    Numeric(NumericBaseType),
    String(&'i str),
    // TODO: the rest
}

#[derive_shrink_wrap]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ItemStruct<'i> {
    pub size: ElementSize,
    pub ident: &'i str,
    pub fields: RefVec<'i, Field<'i>>,
}

#[derive_shrink_wrap]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Field<'i> {
    pub ident: &'i str,
    pub ty: RefBox<'i, Type<'i>>,
    pub default: Option<Value<'i>>,
}
