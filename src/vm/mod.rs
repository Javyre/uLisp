use std::collections::LinkedList;

mod mem;
mod data;

use self::mem::*;
pub use self::data::*;

use std::cell::{RefCell};
use std::rc::Rc;


// pub struct Registers {
//     args: LinkedList<MemData>,
//     rets: LinkedList<MemData>,
// }

pub struct Job {
    mem: Rc<RefCell<Memory>>,
}

pub struct VM {
    // registers: (MemData, MemData, MemData),
    // registers: Registers,
    memory:  Rc<RefCell<Memory>>,
    jobs: Vec<Job>,
}

impl Job {
    pub fn call(&mut self, id: &IdentID) -> MemData {
        let mem = self.mem.clone();

        // FIXME: shouldn't be cloning here
        let insts = mem.borrow().get(0, id).expect("Fetching instructions in scope").as_instructions().unwrap().clone();

        mem.borrow_mut().push_scope();
        let r = self.execute(1, &insts);
        mem.borrow_mut().pop_scope();

        r
    }

    pub fn execute(
        &mut self, mut scope: usize,
        insts: &Instructions) -> MemData {

        let mem = &self.mem;
        let mut register_stack: LinkedList<MemData> = LinkedList::new();
        // let mut scope: usize = scope;

        for inst in insts.iter() {
            println!("{:?}", inst);
            match inst.opcode {
                // OpCode::PSS => { self.memory.inc_scope(1); },
                // OpCode::PPS => { self.memory.dec_scope(1); },

                // FIXME: PSS & PPS should push and pop scope with
                // some id or something for scope
                OpCode::PSS => {
                    mem.borrow_mut().push_scope(); scope += 1;
                },
                OpCode::PPS => {
                    mem.borrow_mut().pop_scope(); scope -= 1;
                },

                OpCode::DFN => {
                    // let is = Instructions::new();
                },
                OpCode::DVR => {
                    let val = inst.val.map_or_else(
                        || register_stack.pop_back().unwrap(),
                        |v| mem.borrow().get_const(&v).unwrap().clone(),
                        );
                    mem.borrow_mut().define(scope,
                                            inst.ident.unwrap(),
                                            val)
                },
                OpCode::LVR => {
                    register_stack.push_back(
                        inst.val.map_or_else(
                            || mem.borrow()
                                  .get(scope, &inst.ident.unwrap())
                                  .unwrap().clone(),
                            |v| mem.borrow()
                                   .get_const(&v)
                                   .unwrap().clone(),
                            )
                        )
                },
                OpCode::CLL => { },
                OpCode::CNV => {
                    let vals = inst.ident
                                   .map_or_else(
                                       || {
                                           let n = register_stack.len() - inst.n.unwrap() as usize;
                                           register_stack.split_off(n)
                                       },
                                       |i| {
                                           let mut ll = LinkedList::new();
                                           ll.push_back(mem.borrow().get(scope, &i).unwrap().clone());
                                           ll
                                       })
                                   .into_iter()
                                   .map(|v| v.convert(&inst.typ.unwrap()));
                    register_stack.extend(vals)
                },
                OpCode::CAT => {
                    let n = register_stack.len() - inst.n.unwrap() as usize;
                    let val = register_stack
                        .split_off(n)
                        .into_iter()
                        .fold(
                            MemData::Str("".to_owned()),
                            |a, s| MemData::Str(format!(
                                    "{}{}",
                                    a.as_string().unwrap(),
                                    s.as_string().unwrap())
                                ));
                    register_stack.push_back(val)
                },
                OpCode::CNS => { },
                OpCode::CAR => { },
                OpCode::CDR => { },
                OpCode::ADD => { },
                OpCode::SUB => { },
                OpCode::MUL => { },
                OpCode::DIV => { },
                OpCode::DSP => {
                    let a = register_stack.pop_back().unwrap();

                    let a = a.as_string().unwrap();

                    println!("{}", a);
                    register_stack.push_back(MemData::Nil);
                },

            }
        }

        assert_eq!(register_stack.len(), 1);
        register_stack.pop_back().unwrap()
    }
}

impl VM {
    pub fn new() -> Self{
        let mem = Rc::new(RefCell::new(Memory::new()));
        Self {
            //registers: Registers::new(),
            memory: mem.clone(),
            jobs: vec![Job { mem }],
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

    pub fn call(&mut self, id: &IdentID) -> MemData {
        self.jobs[0].call(id)
    }

}
