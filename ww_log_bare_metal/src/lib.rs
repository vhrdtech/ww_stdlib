use wire_weaver::prelude::*;

#[ww_trait]
pub trait Log {}

#[derive_shrink_wrap]
#[ww_repr(u4)]
pub enum Severity {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    User(Nibble)
}
