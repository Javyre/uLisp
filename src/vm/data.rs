use super::{
    Error,
    Environment,
};

use std::fmt;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

pub type ConstID = u16;
pub type IdentID = u16;
pub type Quantif = u32; // Actually u18

#[allow(dead_code)]
#[repr(u8)] // actually u6
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum OpCode {
     PSS,
     PPS,
     REC,
     LMB,
     PRC,
     // DFN,
     DVR,
     LVR,
     IFT,
     IFE,
     CGT,
     CLT,
     CEQ,
     CNT,
     CLL,
     CNV,
     CAT,
     CNS,
     CAR,
     CDR,
     ADD,
     SUB,
     MUL,
     DIV,
     DSP,
}

// TODO: make Op compact and outputtable
#[derive(PartialEq, Eq, Clone)]
pub struct Op {
    pub opcode: OpCode, // u6
    pub ident:  Option<IdentID>,
    pub n:      Option<Quantif>, // u18
    pub val:    Option<ConstID>,
    pub typ:    Option<Type>,
    pub mute:   bool,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Type {
    Pointer,
    Lambda,
    Proc,
    Inst,
    Str,
    Pair,
    Int,
    Char,
    Bool,
    Nil,
}

// NOTE: Keep this as small as possible
// pub enum ConstData {
//     Inst(Op),
//     Str(String),
//     Pair { car: ConstID, cdr: ConstID },
//     Int(u32),
//     Char(u8),
//     Bool(bool),
//     Nil,
// }

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MemData {
    // Lambda(Procedure, Stack),
    Pointer(Rc<MemData>),
    Lambda(Procedure, Environment),
    Proc(Procedure),
    Inst(Op),
    Str(String),
    Pair { car: Box<MemData>, cdr: Box<MemData>},
    Int(u32),
    Char(u8),
    Bool(bool),
    Nil, }

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Procedure {
    insts: Vec<Op>
}

pub struct Bin {
    // header: <something>,
    insts: Procedure,
    idents: Vec<IdentID>,
    var_strings: HashMap<IdentID, String>,
    consts: Vec<MemData>, // for now its a simple vec
}


impl Op {
    pub fn new(
        opcode: OpCode,
        ident: Option<IdentID>,
        n: Option<Quantif>,
        val: Option<ConstID>,
        typ: Option<Type>,
        mute: bool) -> Self {

        Self {
            opcode,
            ident,
            n,
            val,
            typ,
            mute,
        }
    }

    // NOTE: ConstID + ofs > u16 = undefined behaviour!!!
    pub fn apply_const_offset(&mut self, ofs: usize) {
        self.val = self.val.map(|cid| cid + ofs as u16); // TODO: make offset actually be able to be usize
    }

    // old and new must be sorted!!
    pub fn apply_ident_swap(&mut self, old: &Vec<IdentID>, new: &Vec<IdentID>) {
        self.ident = self.ident.map(|iid| {
            old.binary_search(&iid).map(|i| *new.get(i).unwrap()).unwrap_or(iid)
        });
    }
}

impl fmt::Debug for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Op(\"{:?}{}{}{}{}{}\")",
               self.opcode,
               self.ident.map_or("".to_owned(), |v| format!(" {:?}", v)),
               self.n.map_or("".to_owned(), |v| format!(" ({:?})", v)),
               self.val.map_or("".to_owned(), |v| format!(" #{:?}", v)),
               self.typ.map_or("".to_owned(), |v| format!(" <{:?}>", v)),
               if self.mute { " &" } else { "" },
               )
    }
}

impl MemData {
    pub fn get_type(&self) -> Type {
        match *self {
            MemData::Pointer(..)=> Type::Pointer,
            MemData::Lambda(..) => Type::Lambda,
            MemData::Proc(..)  => Type::Proc,
            MemData::Inst(..)  => Type::Inst,
            MemData::Str(..)   => Type::Str,
            MemData::Pair {..} => Type::Pair,
            MemData::Int(..)   => Type::Int,
            MemData::Char(..)  => Type::Char,
            MemData::Bool(..)  => Type::Bool,
            MemData::Nil       => Type::Nil,
        }
    }

    #[inline]
    pub fn wrong_type(&self, wanted: Type) -> Error {
        Error::TypeError(wanted, self.get_type())
    }

    /// Traces pointer type back to the source and returns its MemData value
    // pub fn deref(&self) -> Result<Rc<RefCell<MemData>>, &MemData> {
    pub fn deref(&self) -> &MemData {
        if let &MemData::Pointer(ref rc) = self {
            rc.deref()
        } else { 
            &self
        }
    }

    pub fn create_pointer(data: MemData) -> MemData {
        if data.get_type() == Type::Pointer {
            data
        } else {
            MemData::Pointer(Rc::new(data))
        }
    }

    pub fn clone_pointer(&self) -> Option<MemData> {
        if let &MemData::Pointer(ref rc) = self {
            Some(MemData::Pointer(Rc::clone(rc)))
        } else {
            None
        }
    }

    // pub fn as_procedure(&self) -> Result<&Procedure, Error> {
    //     if let &MemData::Lambda(ref i) = self.deref() {
    //         Ok(&i)
    //     } else {
    //         Err(self.wrong_type(Type::Lambda))
    //     }
    // }

    // pub fn as_instruction(&self) -> Result<&Op, Error> {
    //     if let &MemData::Inst(ref o) = self.deref() {
    //         Ok(&o)
    //     } else {
    //         Err(self.wrong_type(Type::Inst))
    //     }
    // }

    // pub fn as_string(&self) -> Result<&String, Error> {
    //     if let &MemData::Str(ref s) = self.deref() {
    //         Ok(&s)
    //     } else {
    //         Err(self.wrong_type(Type::Str))
    //     }
    // }

    // pub fn into_procedure(self) -> Result<Procedure, (Self, Error)> {
    //     if let MemData::Lambda(o) = self {
    //         Ok(o)
    //     } else {
    //         let err = self.wrong_type(Type::Lambda);
    //         Err((self, err))
    //     }
    // }

    // pub fn into_instruction(self) -> Result<Op, (Self, Error)> {
    //     if let MemData::Inst(o) = self {
    //         Ok(o)
    //     } else {
    //         let err = self.wrong_type(Type::Inst);
    //         Err((self, err))
    //     }
    // }

    // pub fn into_pair(self) -> Result<(Box<MemData>, Box<MemData>), (Self, Error)> {
    //     if let MemData::Pair { car, cdr } = self {
    //         Ok((car, cdr))
    //     } else {
    //         let err = self.wrong_type(Type::Pair);
    //         Err((self, err))
    //     }
    // }

    #[inline]
    pub fn is_true(&self) -> bool {
        match *self.deref() {
            MemData::Bool(true) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_false(&self) -> bool {
        match *self.deref() {
            MemData::Bool(false) => true,
            _ => false,
        }
    }

    pub fn cmp(&self, other: &Self) -> Result<Ordering, Error> {
        match (self.deref(), other.deref()) {
        (&MemData::Int(ref s), &MemData::Int(ref o)) => 
            Ok(s.cmp(o)),

        (&MemData::Nil, &MemData::Nil) =>
            Ok(Ordering::Equal),

        _ =>
            Err(Error::BadOperandTypes("ordering", self.get_type(), other.get_type()))
        }
    }

    pub fn gt(&self, other: &Self) -> Result<bool, Error> {
        self.cmp(other).map(|v| v == Ordering::Greater)
    }

    pub fn lt(&self, other: &Self) -> Result<bool, Error> {
        self.cmp(other).map(|v| v == Ordering::Less)
    }

    pub fn eq(&self, other: &Self) -> Result<bool, Error> {
        match (self.deref(), other.deref()) {
            (&MemData::Str(ref s), &MemData::Str(ref o)) =>
                Ok(s == o),

            _ => self.cmp(other).map(|v| v == Ordering::Equal),
        }
    }

    pub fn convert(&self, typ: &Type) -> Result<Self, Error> {
        match *typ {
            Type::Str => {
                Ok(MemData::Str(
                    match *self {
                        MemData::Int(i) => format!("{:?}", i),
                        _ => format!("{:?}", self),
                    }))
            },
            _ => {
                Err(Error::IllegalConversion(self.get_type(), typ.clone()))
            }
        }
    }
}

impl ::std::ops::Add for MemData {
    type Output = Result<MemData, Error>;

    fn add(self, other: Self) -> Self::Output {
        let (a, b) = (self.deref().get_type(), other.deref().get_type());

        // We're dissallowing inter-type operations for now at least
        if a != b { return Err(Error::BadOperandTypes("sum", a, b)) }

        if let (&MemData::Int(a), &MemData::Int(b)) = (self.deref(), other.deref()) {
            return Ok(MemData::Int(a + b))
        }

        Err(Error::BadOperandTypes("sum", a, b))
    }
}

impl ::std::ops::Sub for MemData {
    type Output = Result<MemData, Error>;

    fn sub(self, other: Self) -> Self::Output {
        let (a, b) = (self.deref().get_type(), other.deref().get_type());

        // We're dissallowing inter-type operations for now at least
        if a != b { return Err(Error::BadOperandTypes("subtraction", a, b)) }

        if let (&MemData::Int(a), &MemData::Int(b)) = (self.deref(), other.deref()) {
            return Ok(MemData::Int(a - b))
        }

        Err(Error::BadOperandTypes("subtraction", a, b))
    }
}

impl ::std::ops::Mul for MemData {
    type Output = Result<MemData, Error>;

    fn mul(self, other: Self) -> Self::Output {
        let (a, b) = (self.deref().get_type(), other.deref().get_type());

        // We're dissallowing inter-type operations for now at least
        if a != b { return Err(Error::BadOperandTypes("multiplication", a, b)) }

        if let (&MemData::Int(a), &MemData::Int(b)) = (self.deref(), other.deref()) {
            return Ok(MemData::Int(a * b))
        }

        Err(Error::BadOperandTypes("multiplication", a, b))
    }
}

impl ::std::ops::Div for MemData {
    type Output = Result<MemData, Error>;

    fn div(self, other: Self) -> Self::Output {
        let (a, b) = (self.deref().get_type(), other.deref().get_type());

        // We're dissallowing inter-type operations for now at least
        if a != b { return Err(Error::BadOperandTypes("division", a, b)) }

        if let (&MemData::Int(a), &MemData::Int(b)) = (self.deref(), other.deref()) {
            return Ok(MemData::Int(a / b))
        }

        Err(Error::BadOperandTypes("division", a, b))
    }
}

impl Procedure {
    pub fn apply_const_offset(&mut self, ofs: usize) {
        self.insts.iter_mut().for_each(|i: &mut Op| i.apply_const_offset(ofs));
    }

    pub fn apply_ident_swaps(&mut self, mut old: Vec<IdentID>, mut new: Vec<IdentID>) {
        old.sort_unstable();
        new.sort_unstable();
        self.insts.iter_mut().for_each(|i: &mut Op| i.apply_ident_swap(&old, &new))
    }

    pub fn iter(&self) -> ::std::slice::Iter<Op> {
        self.insts.iter()
    }
}

impl ::std::iter::FromIterator<Op> for Procedure {
    fn from_iter<I: IntoIterator<Item=Op>>(iter: I) -> Self {
        Self {
            insts: iter.into_iter().collect()
        }
    }
}

impl From<Vec<Op>> for Procedure {
    fn from(insts: Vec<Op>) -> Self {
        Self { insts }
    }
}

impl Bin {
    pub fn new(insts: Procedure,
               idents: Vec<IdentID>,
               var_strings: HashMap<IdentID, String>,
               consts: Vec<MemData>) -> Self {
        Self {
            insts,
            idents,
            var_strings,
            consts,
        }
    }

    pub fn unpack(self) -> (Procedure,
                            Vec<IdentID>,
                            HashMap<IdentID, String>,
                            Vec<MemData>) {
        (self.insts, self.idents, self.var_strings, self.consts)
    }

}
