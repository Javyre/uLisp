use std::collections::LinkedList;

#[macro_use]
pub mod macros;

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
    // mem: Rc<RefCell<Memory>>,
    env: Environment,
    // scope: usize,
    // FIXME?: should the self.reg_stack be a Vec<&MemData> instead?
    reg_stack: LinkedList<MemData>,
    recording: usize,
}

pub struct VM {
    // registers: (MemData, MemData, MemData),
    // registers: Registers,
    // memory:  Rc<RefCell<Memory>>,
    consts: Rc<RefCell<Constants>>,
    memory: Environment,
    jobs: Vec<Job>,
}

impl Job {
    // pub fn new(env: Rc<RefCell<Memory>>) -> Self {
    pub fn new(env: Environment) -> Self {
        Self {
            env,
            // scope: 0,
            reg_stack: LinkedList::new(),
            recording: 0,
        }
    }

    pub fn call(&mut self, id: &IdentID) -> Result<MemData, self::RuntimeError> {
        // let mem = self.mem.clone();

        // FIXME: shouldn't be cloning here
        // FIXME: shouldn't be unwrapping here
        // let m = mem.borrow();
        // let insts = m.get(id).unwrap()
        let insts = self.env.get(id).unwrap()
            .as_procedure().expect("Casting memdata as Procedure").clone();
        // drop(m);

        // mem.borrow_mut().push_scope();
        self.env.new_frame();
        self.execute(&insts)?;
        // mem.borrow_mut().pop_scope().unwrap();
        self.env.pop_frame().unwrap();

        self.reg_stack.pop_back().ok_or(self::RuntimeError {
            error: Error::IllegalRegisterPop,
            instruction: None,
            instruction_num: None,
        })
    }

    pub fn run_instruction(&mut self, inst: &Op) -> Result<(), Error> {
        // let mem = self.mem.clone();

        if self.recording > 0 {
            self.recording -= 1;
            self.reg_stack.push_back(MemData::Inst(inst.clone()));
            debug!("recording: {:?}", self.reg_stack.back().unwrap());
            return Ok(());
        }

        debug!("{:?}", inst);
        debug!("{:?}\n", self.reg_stack);
        match inst.opcode {
            // OpCode::PSS => { self.memory.inc_scope(1); },
            // OpCode::PPS => { self.memory.dec_scope(1); },

            // FIXME: PSS & PPS should push and pop scope with
            // some id or something for scope
            OpCode::PSS => {
                // mem.borrow_mut().push_scope(); // self.scope += 1;
                self.env.new_frame();
            },
            OpCode::PPS => {
                // mem.borrow_mut().pop_scope()?; // self.scope -= 1;
                self.env.pop_frame()?;
            },

            OpCode::REC => {
                self.recording = inst.n.unwrap_or(1) as usize;
            },

            // OpCode::DFN => {
            OpCode::LMB => {
                let n = inst.n.expect("getting quatifier");
                let mut is: Vec<Op> = Vec::with_capacity(n as usize);

                let n = self.reg_stack.len() - n as usize;
                for v in self.reg_stack.split_off(n).into_iter() {
                    is.push(v.into_instruction().map_err(|e| e.1)?);
                }

                self.reg_stack.push_back(MemData::Lambda(is.into()))
            },
            OpCode::DVR => {
                let val = if let Some(val) = inst.val {
                    self.env.get_const(&val)?.clone()
                } else {
                    self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?
                };
                // let val = inst.val.map_or_else(
                //     // | | Ok(self.reg_stack.pop_back().expect("popping from register stack")),
                //     | | self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop),
                //     // |v| Ok(mem.borrow().get_const(&v)?.clone()),
                //     |v| Ok(self.env.get_const(&v)?.clone()),
                //     )?;
                // mem.borrow_mut().define(self.scope,
                //                         inst.ident.expect("getting identifier"),
                //                         val)?
                // mem.borrow_mut().define(inst.ident.expect("getting identifier"), val)?
                self.env.define(inst.ident.expect("getting identifier"), val)?
            },
            OpCode::LVR => {
                // let s = self.scope;
                // let m = mem.borrow();
                self.reg_stack.push_back(
                    if let Some(val) = inst.val {
                        self.env.get_const(&val)?.clone()
                    } else {
                        self.env.get(&inst.ident.expect("getting ident"))?.clone()
                    })
                // self.reg_stack.push_back(
                //     inst.val.map_or_else(
                //         | | Ok(self.env.get(&inst.ident.expect("getting ident"))?
                //                 .clone()),
                //         // |v| Ok(mem.borrow().get_const(&v)?.clone()),
                //         |v| Ok(self.env.get_const(&v)?.clone()),
                //         )?
                //     )
            },
            OpCode::IFT | OpCode::IFE => {
                // If-then | If-then-else
                let cond = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;
                let mut fals = None;
                if let OpCode::IFE = inst.opcode {
                    fals = Some(
                        self.reg_stack
                        .pop_back().ok_or(Error::IllegalRegisterPop)?
                        .into_procedure().map_err(|e| e.1)?
                        );
                }
                let tru = self.reg_stack
                    .pop_back().ok_or(Error::IllegalRegisterPop)?
                    .into_procedure().map_err(|e| e.1)?;

                // let sc = self.scope;
                if !cond.is_false() {
                    self.execute(&tru)
                        .map_err(|e| Error::RuntimeErrorInSubJob(Box::new(e)))?
                } else if let OpCode::IFE = inst.opcode {
                    self.execute(&fals.unwrap())
                        .map_err(|e| Error::RuntimeErrorInSubJob(Box::new(e)))?
                }
            },
            OpCode::CGT | OpCode::CLT | OpCode::CEQ => {
                // Cond ordering
                let n = self.reg_stack.len() - inst.n.expect("getting quantifier") as usize;
                let mut iter = self.reg_stack.split_off(n)
                    .into_iter()
                    .peekable();

                let mut r = true;
                while let Some(ref v) = iter.next() {
                    r = r && if let Some(n) = iter.peek() {
                        match inst.opcode {
                            OpCode::CGT     => { (v.gt(n))?  },
                            OpCode::CLT     => { (v.lt(n))?  },
                            OpCode::CEQ | _ => { (v == n)  },
                        }
                    } else { true };
                }
                self.reg_stack.push_back(MemData::Bool(r))
            },
            OpCode::CNT => {
                // Cond NOT
            },
            OpCode::CLL => {
                let len = self.reg_stack.len();
                // let m = mem.borrow();
                let insts: Procedure = if let Some(i) = inst.ident {
                                // FIXME: should be cloning here!!!
                                self.env.get(&i)?.as_procedure()?.clone()
                            } else {
                                let n = inst.n.expect("getting quantifier");
                                let mut insts = Vec::with_capacity(n as usize);
                                let n = len - n as usize;
                                for v in self.reg_stack.split_off(n).into_iter() {
                                    insts.push(v.into_instruction().map_err(|e| e.1)?)
                                }
                                insts.into()
                            };
                // drop(m);

                self.env.new_frame();
                trace!("Entering subjob!");
                // let s = self.scope + 1;
                self.execute(&insts)
                    .map_err(|e| { trace!("RUNTIME ERROR!"); e } )
                    .map_err(|e| Error::RuntimeErrorInSubJob(Box::new(e)))?;
                let r = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;
                trace!("Done subjob!");
                self.env.pop_frame().unwrap();

                self.reg_stack.push_back(r)

            },
            OpCode::CNV => {
                // let s = self.scope;
                // let m = mem.borrow();
                let vals = if let Some(ident) = inst.ident {
                    let mut ll = LinkedList::new();
                    ll.push_back(self.env.get(&ident)?.clone());
                    ll
                } else {
                    let n = self.reg_stack.len() - inst.n.unwrap() as usize;
                    self.reg_stack.split_off(n)
                }.into_iter().map(|v| v.convert(&inst.typ.unwrap()));

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
            OpCode::CNS => {
                let n = self.reg_stack.len() - 2;
                let mut pair = self.reg_stack.split_off(n).into_iter();
                let car = Box::new(pair.next().unwrap());
                let cdr = Box::new(pair.next().unwrap());
                self.reg_stack.push_back(MemData::Pair { car, cdr })
            },
            OpCode::CAR | OpCode::CDR => {
                let vals = if let Some(i) = inst.ident {
                    let mut r = LinkedList::new();
                    // let m = mem.borrow();
                    r.push_back(self.env.get(&i)?.deref().clone());
                    r
                } else {
                    let n = self.reg_stack.len() - inst.n.unwrap_or(1) as usize;
                    self.reg_stack.split_off(n)
                };

                let mut r = LinkedList::new();
                for v in vals.into_iter() {
                    r.push_back(
                        match inst.opcode {
                            OpCode::CAR => {
                                *v.into_pair().map_err(|e| e.1)?.0
                            }
                            OpCode::CDR | _ => {
                                *v.into_pair().map_err(|e| e.1)?.1
                            }
                        }
                        )
                }

                self.reg_stack.append(&mut r)
            },

            | OpCode::ADD | OpCode::SUB
            | OpCode::MUL | OpCode::DIV => {
                let vals = if let Some(i) = inst.ident {
                    let mut r = LinkedList::new();
                    // let m = mem.borrow();
                    r.push_back(self.env.get(&i)?.clone());
                    r.push_back(self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?);
                    r
                } else {
                    let n = self.reg_stack.len() - inst.n.unwrap_or(1) as usize;
                    self.reg_stack.split_off(n)
                };
                self.reg_stack.push_back(
                    if vals.len() == 0 {
                        MemData::Nil
                    } else {
                        let mut vals = vals.into_iter();
                        let r = vals.next().unwrap();
                        match inst.opcode {
                            OpCode::ADD =>     vals.fold(Ok(r), |a, v| Ok((a? + v)?) )?,
                            OpCode::SUB =>     vals.fold(Ok(r), |a, v| Ok((a? - v)?) )?,
                            OpCode::MUL =>     vals.fold(Ok(r), |a, v| Ok((a? * v)?) )?,
                            OpCode::DIV | _ => vals.fold(Ok(r), |a, v| Ok((a? / v)?) )?,
                        }
                    })
            },
            OpCode::DSP => {
                let a = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;

                let a = a.as_string()?;

                print!("{}", a);
                if ! inst.mute {
                    self.reg_stack.push_back(MemData::Nil);
                }
            },

        }
        Ok(())
    }

    pub fn execute(
        &mut self,
        insts: &Procedure) -> Result<(), err::RuntimeError> {

        let initial_len = self.reg_stack.len();
        // let mut register_stack: LinkedList<MemData> = LinkedList::new();
        // let mut scope: usize = scope;
        // let old_scope = self.scope;
        // self.scope = scope;

        for (i, inst) in insts.iter().enumerate() {
            self.run_instruction(&inst)
                .map_err(|e| RuntimeError {
                    instruction: Some(inst.clone()),
                    instruction_num: Some(i),
                    error: e
                })?
                // }).unwrap()
        }

        // self.scope = old_scope;

        debug!("final: {:?}", self.reg_stack);
        // assert_eq!(self.reg_stack.len(), initial_len + 1);
        // Ok(self.reg_stack.pop_back().unwrap())
        Ok(())
    }
}

impl VM {
    pub fn new() -> Self{
        // let mem = Rc::new(RefCell::new(Memory::new()));
        // let mem = Rc::new(RefCell::new(Environment::new()));
        let consts = Rc::new(RefCell::new(Vec::new()));
        let mem =  Environment::new(consts.clone());

        Self {
            //registers: Registers::new(),
            consts: consts,
            memory: mem.clone(),
            jobs: vec![Job::new(mem)],
        }
    }

    // return IdentID of the function representing the bin
    pub fn load(&mut self, bin: Bin) -> IdentID {
        let (mut insts, idents, mut var_strings, consts) = bin.unpack();
        // let const_ofs = self.memory.borrow_mut().load_consts(consts);
        let const_ofs = {
            let o = self.memory.max_const_id().map_or(0, |i| i+1);
            consts.into_iter().for_each(|v| { self.memory.load_const(v); } );
            o
        };

        // let (idents, _) = self.memory.borrow_mut().bind_idents(idents, var_strings);
        let new_idents = idents.clone().into_iter().map(|i| {
            self.memory.new_ident_id(var_strings.remove(&i))
        } ).collect();

        insts.apply_ident_swaps(idents, new_idents);

        insts.apply_const_offset(const_ofs);
        // let id = self.memory.borrow_mut().bind_ident_id(None);
        // self.memory.borrow_mut().define(id, MemData::Lambda(insts));

        let id = self.memory.new_ident_id(None);
        self.memory.define(id, MemData::Lambda(insts));

        id
    }

    pub fn call(&mut self, id: &IdentID) -> Result<MemData, self::RuntimeError> {
        self.jobs[0].call(id)
    }

}
