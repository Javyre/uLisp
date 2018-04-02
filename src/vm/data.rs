use super::Error;

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
     // DFN,
     DVR,
     LVR,
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
#[derive(PartialEq, Eq, Debug, Clone)]
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
    Lambda,
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
    Lambda(Instructions),
    Inst(Op),
    Str(String),
    Pair { car: Box<MemData>, cdr: Box<MemData>},
    Int(u32),
    Char(u8),
    Bool(bool),
    Nil,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Instructions {
    insts: Vec<Op>
}

pub struct Bin {
    // header: <something>,
    insts: Instructions,
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
        self.val.map(|cid| cid + ofs as u16); // TODO: make offset actually be able to be usize
    }
}

impl MemData {
    pub fn get_type(&self) -> Type {
        match *self {
            MemData::Lambda(..) => Type::Lambda,
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
    fn wrong_type(&self, wanted: Type) -> Error {
        Error::TypeError(wanted, self.get_type())
    }

    pub fn as_instructions(&self) -> Result<&Instructions, Error> {
        if let &MemData::Lambda(ref i) = self {
            Ok(&i)
        } else {
            Err(self.wrong_type(Type::Lambda))
        }
    }

    pub fn as_instruction(&self) -> Result<&Op, Error> {
        if let &MemData::Inst(ref o) = self {
            Ok(&o)
        } else {
            Err(self.wrong_type(Type::Inst))
        }
    }

    pub fn as_string(&self) -> Result<&String, Error> {
        if let &MemData::Str(ref s) = self {
            Ok(&s)
        } else {
            Err(self.wrong_type(Type::Str))
        }
    }

    pub fn into_instruction(self) -> Result<Op, (Self, Error)> {
        if let MemData::Inst(o) = self {
            Ok(o)
        } else {
            let err = self.wrong_type(Type::Inst);
            Err((self, err))
        }
    }

    pub fn into_pair(self) -> Result<(Box<MemData>, Box<MemData>), (Self, Error)> {
        if let MemData::Pair { car, cdr } = self {
            Ok((car, cdr))
        } else {
            let err = self.wrong_type(Type::Pair);
            Err((self, err))
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
        let (a, b) = (self.get_type(), other.get_type());

        // We're dissallowing inter-type operations for now at least
        if a != b { return Err(Error::BadOperandTypes("sum", a, b)) }

        if let (MemData::Int(a), MemData::Int(b)) = (self, other) {
            return Ok(MemData::Int(a + b))
        }

        Err(Error::BadOperandTypes("sum", a, b))
    }
}

impl ::std::ops::Sub for MemData {
    type Output = Result<MemData, Error>;

    fn sub(self, other: Self) -> Self::Output {
        let (a, b) = (self.get_type(), other.get_type());

        // We're dissallowing inter-type operations for now at least
        if a != b { return Err(Error::BadOperandTypes("subtraction", a, b)) }

        if let (MemData::Int(a), MemData::Int(b)) = (self, other) {
            return Ok(MemData::Int(a - b))
        }

        Err(Error::BadOperandTypes("subtraction", a, b))
    }
}

impl ::std::ops::Mul for MemData {
    type Output = Result<MemData, Error>;

    fn mul(self, other: Self) -> Self::Output {
        let (a, b) = (self.get_type(), other.get_type());

        // We're dissallowing inter-type operations for now at least
        if a != b { return Err(Error::BadOperandTypes("multiplication", a, b)) }

        if let (MemData::Int(a), MemData::Int(b)) = (self, other) {
            return Ok(MemData::Int(a * b))
        }

        Err(Error::BadOperandTypes("multiplication", a, b))
    }
}

impl ::std::ops::Div for MemData {
    type Output = Result<MemData, Error>;

    fn div(self, other: Self) -> Self::Output {
        let (a, b) = (self.get_type(), other.get_type());

        // We're dissallowing inter-type operations for now at least
        if a != b { return Err(Error::BadOperandTypes("division", a, b)) }

        if let (MemData::Int(a), MemData::Int(b)) = (self, other) {
            return Ok(MemData::Int(a / b))
        }

        Err(Error::BadOperandTypes("division", a, b))
    }
}

impl Instructions {
    pub fn apply_const_offset(&mut self, ofs: usize) {
        self.insts.iter_mut().for_each(|i: &mut Op| i.apply_const_offset(ofs));
    }

    pub fn iter(&self) -> ::std::slice::Iter<Op> {
        self.insts.iter()
    }
}

impl ::std::iter::FromIterator<Op> for Instructions {
    fn from_iter<I: IntoIterator<Item=Op>>(iter: I) -> Self {
        Self {
            insts: iter.into_iter().collect()
        }
    }
}

impl From<Vec<Op>> for Instructions {
    fn from(insts: Vec<Op>) -> Self {
        Self { insts }
    }
}

impl Bin {
    pub fn new(insts: Instructions, consts: Vec<MemData>) -> Self {
        Self {
            insts,
            consts,
        }
    }

    pub fn unpack(self) -> (Instructions, Vec<MemData>) {
        (self.insts, self.consts)
    }

}
