use std::mem;

use super::{scope::SignatureArgument, Data};

macro_rules! signature_fn {
    ($name:ident, $arg:expr) => {
        pub fn $name() -> SignatureArgument {
            SignatureArgument::Data(mem::discriminant(&$arg))
        }
    };
}

pub fn raw() -> SignatureArgument {
    SignatureArgument::Raw
}

pub fn any() -> SignatureArgument {
    SignatureArgument::Any
}

signature_fn!(string, Data::String("".to_string()));
signature_fn!(number, Data::Number(0.));
signature_fn!(boolean, Data::Boolean(false));
