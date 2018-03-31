use std::fmt;
use super::{
    Type,
    IdentID,
    ConstID,
    Op,
};

#[derive(Debug)]
pub struct RuntimeError {
    pub instruction: Op,
    pub instruction_num: usize,
    pub error: Error,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "an error occured during execution of job: instruction(#{}): `{:?}`: {}",
            self.instruction_num, self.instruction, self.error)
    }
}

impl ::std::error::Error for RuntimeError {
    fn description(&self) -> &str {
            "an error occured during execution of job"
    }
}

#[derive(Debug)]
pub enum Error {
    TypeError(Type, Type),
    VariableNotFound(usize, IdentID),
    ConstantNotFound(ConstID),
    IllegalStackPop,
    IllegalRegisterPop,
    IllegalConversion(Type, Type),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::TypeError(ref a, ref b) =>
                write!(f, "expected type `{:?}` but found `{:?}`", a, b),
            Error::VariableNotFound(ref scope, ref id) =>
                write!(f, "variable not found in scope `{}`: {:?}", scope, id),
            Error::ConstantNotFound(ref id) =>
                write!(f, "constant not found: {:?}", id),
            Error::IllegalStackPop =>
                write!(f, "illegal stack frame pop: already in root scope!"),
            Error::IllegalRegisterPop =>
                write!(f, "illegal register stack pop: not enough items in register stack to pop!"),
            Error::IllegalConversion(ref a, ref b) =>
                write!(f, "illegal conversion target: from `{:?}` to `{:?}`", a, b),
        }
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::TypeError(..)         => "unexpected type found",
            Error::VariableNotFound(..)  => "variable not found in scope",
            Error::ConstantNotFound(..)  => "constant not found",
            Error::IllegalStackPop       => "illegal stack frame pop",
            Error::IllegalRegisterPop    => "illegal register stack pop",
            Error::IllegalConversion(..) => "illegal conversion target"
        }
    }
}
