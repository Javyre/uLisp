use std::collections::LinkedList;

mod mem;

use self::mem::*;
use std::cell::{RefCell};

pub type ConstID = u16;
pub type IdentID = u16;
pub type Quantif = u32; // Actually u18

pub struct Registers {
    args: LinkedList<MemData>,
    rets: LinkedList<MemData>,
}

#[repr(u8)] // actually u6
#[derive(Clone)]
pub enum OpCode {
     PSS,
     PPS,
     DFN,
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
#[derive(Clone)]
pub struct Op {
    pub opcode: OpCode, // u6
    pub ident:  Option<IdentID>,
    pub n:      Option<Quantif>, // u18
    pub val:    Option<ConstID>,
    pub typ:    Option<Type>,
    pub mute:   bool,
}

#[repr(u8)]
#[derive(Clone)]
pub enum Type {
    Insts,
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

#[derive(Clone)]
pub enum MemData {
    Insts(Instructions),
    Inst(Op),
    Str(String),
    Pair { car: Box<MemData>, cdr: Box<MemData>},
    Int(u32),
    Char(u8),
    Bool(bool),
    Nil,
}

#[derive(Clone)]
pub struct Instructions {
    insts: Vec<Op>
}

pub struct Bin {
    // header: <something>,
    insts: Instructions,
    consts: Vec<MemData>, // for now its a simple vec
}

pub struct VM {
    // registers: (MemData, MemData, MemData),
    registers: Registers,
    memory:  RefCell<Memory>,
    // jobs: Vec<Job>,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            args: LinkedList::new(),
            rets: LinkedList::new(),
        }
    }

    pub fn args_to_rets(&mut self, n: usize) {
        self.rets.append(&mut self.args.split_off(n))
    }

    pub fn rets_to_args(&mut self, n: usize) {
        self.args.append(&mut self.rets.split_off(n))
    }
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

    pub fn into_raw(self) -> MemData {
        unimplemented!()
    }

    // NOTE: ConstID + ofs > u16 = undefined behaviour!!!
    pub fn apply_const_offset(&mut self, ofs: usize) {
        self.val.map(|cid| cid + ofs as u16); // TODO: make offset actually be able to be usize
    }
}


// impl Into<Vec<MemData>> for Vec<ConstData> {
//     fn into(self) -> MemData {
//         match self {
//             ConstData::Inst(x) => MemData::Inst(x),
//             ConstData::Str(x) => MemData::Str(x),
//             ConstData::Pair { car, cdr } => MemData::Pair { car: Box::new(), cdr: Box::new() },
//             ConstData::Int(x)  => MemData::Int(x),
//             ConstData::Char(x) => MemData::Char(x),
//             ConstData::Bool(x) => MemData::Bool(x),
//             ConstData::Nil => MemData::Nil,
//         }
//     }
// }

impl MemData {
    pub fn as_instructions(&self) -> Option<&Instructions> {
        if let &MemData::Insts(ref i) = self {
            Some(&i)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let &MemData::Str(ref s) = self {
            Some(&s)
        } else {
            None
        }
    }
}

impl Instructions {
    pub fn into_raw(self) -> Vec<MemData> {
        return self.insts.into_iter().map(|i| i.into_raw()).collect()
    }

    pub fn apply_const_offset(&mut self, ofs: usize) {
        self.insts.iter_mut().for_each(|i: &mut Op| i.apply_const_offset(ofs));
    }

    pub fn iter(&self) -> ::std::slice::Iter<Op> {
        self.insts.iter()
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

impl VM {
    pub fn new() -> Self{
        Self {
            registers: Registers::new(),
            memory: RefCell::new(Memory::new()),
            // jobs: vec![],
        }
    }

    // return IdentID of the function representing the bin
    pub fn load(&mut self, bin: Bin) -> IdentID {
        let (mut insts, consts) = bin.unpack();
        let const_ofs = self.memory.borrow().load_consts(consts);

        insts.apply_const_offset(const_ofs);
        let id = self.memory.borrow().generate_ident_id(0);
        self.memory.borrow_mut().define(0, id, MemData::Insts(insts));

        id
    }

    pub fn call(&mut self, id: &IdentID) -> MemData {
        let insts = self.memory.borrow().get(0, id).unwrap().as_instructions().unwrap();

        self.execute(1, insts)
    }

    pub fn execute(&mut self, mut scope: usize, insts: &Instructions) -> MemData {
        let mut register_stack: Vec<MemData> = Vec::new();
        // let mut scope: usize = scope;

        for inst in insts.iter() {
            match inst.opcode {
                // OpCode::PSS => { self.memory.inc_scope(1); },
                // OpCode::PPS => { self.memory.dec_scope(1); },
                OpCode::PSS => { scope += 1; },
                OpCode::PPS => { scope -= 1; },

                OpCode::DFN => {
                    // let is = Instructions::new();
                },
                OpCode::DVR => {
                    self.memory.borrow_mut().define(
                        scope,
                        inst.ident.unwrap(),
                        inst.val.map_or_else(
                            || register_stack.pop().unwrap(),
                            |v| self.memory.borrow().get_const(&v).unwrap().clone(),
                            )
                        )
                },
                OpCode::LVR => {
                    register_stack.push(
                        inst.val.map_or_else(
                            || self.memory.borrow().get(scope, &inst.ident.unwrap()).unwrap().clone(),
                            |v| self.memory.borrow().get_const(&v).unwrap().clone(),
                            )
                        )
                },
                OpCode::CLL => { },
                OpCode::CNV => { },
                OpCode::CAT => { },
                OpCode::CNS => { },
                OpCode::CAR => { },
                OpCode::CDR => { },
                OpCode::ADD => { },
                OpCode::SUB => { },
                OpCode::MUL => { },
                OpCode::DIV => { },
                OpCode::DSP => {
                    println!("{}", register_stack.pop().unwrap().as_string().unwrap());
                    register_stack.push(MemData::Nil);
                },

            }
        }

        assert_eq!(register_stack.len(), 1);
        register_stack.pop().unwrap()
    }
}
