use std::mem;

use super::{scope::{SignatureArgument, ReturnType}, Data, ControlFlow};

macro_rules! signature_arg {
    ($name:ident, $arg:expr) => {
        pub fn $name() -> SignatureArgument {
            SignatureArgument::Data(mem::discriminant(&$arg))
        }
    };
}

macro_rules! return_type {
    ($name:ident, $arg:expr) => {
        pub fn $name() -> ReturnType {
            ReturnType::Data(mem::discriminant(&$arg))
        }
    };
}

pub fn arg_raw() -> SignatureArgument {
    SignatureArgument::Raw
}

pub fn arg_any() -> SignatureArgument {
    SignatureArgument::Any
}

signature_arg!(arg_string, Data::String("".to_string()));
signature_arg!(arg_number, Data::Number(0.));
signature_arg!(arg_boolean, Data::Boolean(false));
signature_arg!(arg_list, Data::List(Vec::new()));


pub fn return_any() -> ReturnType {
    ReturnType::Any
}

return_type!(return_string, Data::String("".to_string()));
return_type!(return_number, Data::Number(0.));
return_type!(return_boolean, Data::Boolean(false));
return_type!(return_list, Data::List(Vec::new()));
return_type!(return_control, Data::ControlFlow(ControlFlow::Break));
return_type!(return_unit, Data::Unit);