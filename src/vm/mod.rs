use std::collections::LinkedList;

mod mem;
mod data;
mod err;

use self::mem::*;
pub use self::data::*;
pub use self::err::*;

use std::cell::{RefCell};
use std::rc::Rc;


// pub struct Registers {
//     args: LinkedList<MemData>,
//     rets: LinkedList<MemData>,
// }

pub struct Job {
    mem: Rc<RefCell<Memory>>,
    scope: usize,
    reg_stack: LinkedList<MemData>,
}

pub struct VM {
    // registers: (MemData, MemData, MemData),
    // registers: Registers,
    memory:  Rc<RefCell<Memory>>,
    jobs: Vec<Job>,
}

impl Job {
    pub fn new(mem: Rc<RefCell<Memory>>) -> Self {
        Self {
            mem,
            scope: 0,
            reg_stack: LinkedList::new(),
        }
    }

    pub fn call(&mut self, id: &IdentID) -> Result<MemData, self::RuntimeError> {
        let mem = self.mem.clone();

        // FIXME: shouldn't be cloning here
        // FIXME: shouldn't be unwrapping here
        let insts = mem.borrow().get(0, id).unwrap()
            .as_instructions().expect("Casting memdata as Instructions").clone();

        mem.borrow_mut().push_scope();
        let r = self.execute(1, &insts);
        mem.borrow_mut().pop_scope().unwrap();

        r
    }

    pub fn run_instruction(&mut self, inst: &Op) -> Result<(), Error> {
        let mem = self.mem.clone();

        println!("{:?}", inst);
        match inst.opcode {
            // OpCode::PSS => { self.memory.inc_scope(1); },
            // OpCode::PPS => { self.memory.dec_scope(1); },

            // FIXME: PSS & PPS should push and pop scope with
            // some id or something for scope
            OpCode::PSS => {
                mem.borrow_mut().push_scope(); self.scope += 1;
            },
            OpCode::PPS => {
                mem.borrow_mut().pop_scope()?; self.scope -= 1;
            },

            OpCode::DFN => {
                // let is = Instructions::new();
            },
            OpCode::DVR => {
                let val = inst.val.map_or_else(
                    // | | Ok(self.reg_stack.pop_back().expect("popping from register stack")),
                    | | self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop),
                    |v| Ok(mem.borrow().get_const(&v)?.clone()),
                    )?;
                mem.borrow_mut().define(self.scope,
                                        inst.ident.expect("getting quatifier"),
                                        val)
            },
            OpCode::LVR => {
                let s = self.scope;
                self.reg_stack.push_back(
                    inst.val.map_or_else(
                        | | Ok(mem.borrow()
                              .get(s, &inst.ident.expect("getting ident"))?
                              .clone()),
                        |v| Ok(mem.borrow().get_const(&v)?.clone()),
                        )?
                    )
            },
            OpCode::CLL => { },
            OpCode::CNV => {
                let s = self.scope;
                let vals = inst.ident
                               .map_or_else(
                                   | | Ok({
                                       let n = self.reg_stack.len() - inst.n.unwrap() as usize;
                                       self.reg_stack.split_off(n)
                                   }),
                                   |i| Ok({
                                       let mut ll = LinkedList::new();
                                       ll.push_back(mem.borrow().get(s, &i)?.clone());
                                       ll
                                   }))?
                               .into_iter()
                               .map(|v| v.convert(&inst.typ.unwrap()));
                for v in vals {
                    self.reg_stack.push_back(v?)
                }
                // self.reg_stack.extend(vals)
            },
            OpCode::CAT => {
                let n = self.reg_stack.len() - inst.n.unwrap() as usize;
                let vals = self.reg_stack.split_off(n);
                let mut val = "".to_owned();
                for v in vals {
                    val = format!("{}{}", val, v.as_string()?)
                }
                self.reg_stack.push_back(MemData::Str(val))
            },
            OpCode::CNS => { },
            OpCode::CAR => { },
            OpCode::CDR => { },
            OpCode::ADD => { },
            OpCode::SUB => { },
            OpCode::MUL => { },
            OpCode::DIV => { },
            OpCode::DSP => {
                let a = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;

                let a = a.as_string()?;

                print!("{}", a);
                self.reg_stack.push_back(MemData::Nil);
            },

        }
        Ok(())
    }

    pub fn execute(
        &mut self, scope: usize,
        insts: &Instructions) -> Result<MemData, err::RuntimeError> {

        let initial_len = self.reg_stack.len();
        // let mut register_stack: LinkedList<MemData> = LinkedList::new();
        // let mut scope: usize = scope;
        self.scope = scope;

        for (i, inst) in insts.iter().enumerate() {
            self.run_instruction(&inst)
                .map_err(|e| RuntimeError {
                    instruction: inst.clone(),
                    instruction_num: i,
                    error: e
                })?
        }

        assert_eq!(self.reg_stack.len(), initial_len + 1);
        Ok(self.reg_stack.pop_back().unwrap())
    }
}

impl VM {
    pub fn new() -> Self{
        let mem = Rc::new(RefCell::new(Memory::new()));
        Self {
            //registers: Registers::new(),
            memory: mem.clone(),
            jobs: vec![Job::new(mem)],
        }
    }

    // return IdentID of the function representing the bin
    pub fn load(&mut self, bin: Bin) -> IdentID {
        let (mut insts, consts) = bin.unpack();
        let const_ofs = self.memory.borrow_mut().load_consts(consts);

        insts.apply_const_offset(const_ofs);
        let id = self.memory.borrow_mut().generate_ident_id(0);
        self.memory.borrow_mut().define(0, id, MemData::Insts(insts));

        id
    }

    pub fn call(&mut self, id: &IdentID) -> Result<MemData, self::RuntimeError> {
        self.jobs[0].call(id)
    }

}
