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


bitflags! {
    pub struct LoadOpts: u8 {
        const OVERRIDE_VAR_STRINGS = 0b00000001;
        const REUSE_VAR_STRINGS    = 0b00000010;
        const DEFAULTS             = Self::OVERRIDE_VAR_STRINGS.bits;
    }
}

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
        let mut env = None;
        let insts = (|| {
            // FIXME: should be cloning here!!!
            let v = self.env.get(&id)?;
            Ok(match v.deref() {
                &MemData::Proc(ref p) => p.clone(),
                &MemData::Lambda(ref p, ref e) => {
                    env = Some(e.clone());
                    p.clone()
                },
                _ => Err(v.wrong_type(Type::Proc))?,
            })
        })().map_err(|e| self::RuntimeError {
            error: e,
            instruction: None,
            instruction_num: None,
        })?;

        self.env.new_frame();
        trace!("Entering subjob!");

        self.execute(&insts, env)
            .map_err(|e| { trace!("RUNTIME ERROR!"); e } )?;
        (|| {
            let r = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;
            trace!("Done subjob!");
            self.env.pop_frame().unwrap();

            Ok(r)
        })().map_err(|e| self::RuntimeError {
            error: e,
            instruction: None,
            instruction_num: None,
        })
    }

    pub fn run_instruction(&mut self, inst: &Op) -> Result<(), Error> {
        if self.recording > 0 {
            self.recording -= 1;
            self.reg_stack.push_back(MemData::Inst(inst.clone()));
            debug!("recording: {:?}", self.reg_stack.back().unwrap());
            return Ok(());
        }

        debug!("{:?}", inst);
        debug!("{:?}\n", self.reg_stack);
        match inst.opcode {
            OpCode::PSS => {
                self.env.new_frame();
            },
            OpCode::PPS => {
                self.env.pop_frame()?;
            },

            OpCode::REC => {
                self.recording = inst.n.unwrap_or(1) as usize;
            },

            OpCode::LMB | OpCode::PRC => {
                let n = inst.n.expect("getting quatifier");
                let mut is: Vec<Op> = Vec::with_capacity(n as usize);

                let n = self.reg_stack.len() - n as usize;
                for v in self.reg_stack.split_off(n).into_iter() {
                    map_as!(v => Inst(i) => is.push(i))?;
                }

                self.reg_stack.push_back(
                    match inst.opcode {
                        OpCode::LMB => MemData::Lambda(is.into(), self.env.clone()),
                        OpCode::PRC | _ => MemData::Proc(is.into()),
                    })
            },
            OpCode::DVR => {
                let val = if let Some(val) = inst.val {
                    self.env.get_const(&val)?.clone()
                } else {
                    self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?
                };
                self.env.define(inst.ident.expect("getting identifier"), val)?; 

                if ! inst.mute {
                    self.reg_stack.push_back(self.env.get(&inst.ident.unwrap()).unwrap());
                }
            },
            OpCode::LVR => {
                self.reg_stack.push_back(
                    if let Some(val) = inst.val {
                        self.env.get_const(&val)?.clone()
                    } else {
                        self.env.get(&inst.ident.expect("getting ident"))?.clone()
                    })
            },
            OpCode::IFT | OpCode::IFE => {
                // If-then | If-then-else
                let cond = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;

                let mut fals = None;
                if let OpCode::IFE = inst.opcode {
                    let v = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;
                    fals = Some(map_as!(*v.deref() => Proc(ref p) => p.clone())?);
                }

                let v = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;
                let tru = map_as!(*v.deref() => Proc(ref p) => p.clone())?;

                if !cond.is_false() {
                    self.execute(&tru, None)
                        .map_err(|e| Error::RuntimeErrorInSubJob(Box::new(e)))?
                } else if let OpCode::IFE = inst.opcode {
                    self.execute(&fals.unwrap(), None)
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
                let r = if let Some(i) = inst.ident {
                    self.call(&i)
                        .map_err(|e| Error::RuntimeErrorInSubJob(Box::new(e)))?
                } else {
                    // setup
                    let len = self.reg_stack.len();
                    let n = inst.n.expect("getting quantifier");
                    let mut insts = Vec::with_capacity(n as usize);

                    // prepare instrucitons list
                    let n = len - n as usize;
                    for v in self.reg_stack.split_off(n).into_iter() {
                        map_as!(v => Inst(o) => insts.push(o) )?;
                    }
                    let insts = insts.into();

                    // execute
                    self.env.new_frame();
                    trace!("Entering subjob!");
                    self.execute(&insts, None)
                        .map_err(|e| { trace!("RUNTIME ERROR!"); e } )
                        .map_err(|e| Error::RuntimeErrorInSubJob(Box::new(e)))?;
                    let r = self.reg_stack.pop_back().ok_or(Error::IllegalRegisterPop)?;
                    trace!("Done subjob!");
                    self.env.pop_frame().unwrap();

                    r
                };
                self.reg_stack.push_back(r)

            },
            OpCode::CNV => {
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
            },
            OpCode::CAT => {
                let n = self.reg_stack.len() - inst.n.unwrap() as usize;
                let vals = self.reg_stack.split_off(n);
                let mut val = "".to_owned();
                for v in vals {
                    map_as!(*v.deref() => Str(ref s) => {
                        val = format!("{}{}", val, s)
                    })?;
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
                    map_as!(v => Pair { car, cdr } => 
                            r.push_back(
                                *match inst.opcode {
                                    OpCode::CAR     => car,
                                    OpCode::CDR | _ => cdr,
                                }))?;
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

                let a = map_as!(*a.deref() => Str(ref s) => s)?;

                print!("{}", a);
                trace!("DISPLAY: {}", a);
                if ! inst.mute {
                    self.reg_stack.push_back(MemData::Nil);
                }
            },

        }
        Ok(())
    }

    pub fn execute(
        &mut self,
        insts: &Procedure,
        env: Option<Environment>) -> Result<(), err::RuntimeError> {

        // BEGIN Env swapping
        let mut old_env = None;
        // let env = env.map(
        //     |e| {
        //         let mut ee = self.env.clone();
        //         ee.append(e);
        //         ee
        //     });
        if let Some(env) = env {
            old_env = Some(::std::mem::replace(&mut self.env, env));
        }
        // END Env swapping

        let initial_len = self.reg_stack.len();

        for (i, inst) in insts.iter().enumerate() {
            self.run_instruction(&inst)
                .map_err(|e| RuntimeError {
                    instruction: Some(inst.clone()),
                    instruction_num: Some(i),
                    error: e
                })?
        }

        // BEGIN Env restore
        if let Some(old_env) = old_env {
            let _ = ::std::mem::replace(&mut self.env, old_env);
        }
        // END Env restore

        debug!("final: {:?}", self.reg_stack);
        Ok(())
    }
}

impl VM {
    pub fn new() -> Self{
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
    pub fn load(&mut self, bin: Bin, flags: LoadOpts) -> IdentID {
        // early reserve our ident for proper offsets
        let id = self.memory.new_ident_id(None);
        self.memory.define(id, MemData::Nil).unwrap();

        let (mut insts, idents, mut var_strings, consts) = bin.unpack();
        let const_ofs = {
            let o = self.memory.max_const_id().map_or(0, |i| i+1);
            consts.into_iter().for_each(|v| { self.memory.load_const(v); } );
            o
        };

        let mut ii = self.memory.new_ident_id(None) - 1;
        let new_idents = idents.clone().into_iter().map(|i| {
            ii+=1;
            if flags.contains(LoadOpts::REUSE_VAR_STRINGS) {
                if let Some(s) = var_strings.get(&i) {
                    if let Some(id) = self.memory.get_ident(s) {
                        trace!("Reusing id: {} with var_str: {}", id, s);
                        id
                    } else {
                        trace!("Creating new var_str: {} with id: {}", s, id);
                        self.memory.bind_var_string(s.to_owned(), i+ii);
                        i+ii
                    }
                } else {
                    i+ii
                }
            } else {
                if let Some(s) = var_strings.get(&i) {
                    trace!("Creating new var_str: {} with id: {}", s, id);
                    self.memory.bind_var_string(s.to_owned(), i+ii);
                }
                i+ii
            }
            // self.memory.new_ident_id(var_strings.remove(&i))
        } ).collect();

        insts.apply_ident_swaps(idents, new_idents);
        insts.apply_const_offset(const_ofs);

        let env = self.memory.clone();
        self.memory.define(id, MemData::Lambda(insts, env));

        id
    }

    pub fn call(&mut self, id: &IdentID) -> Result<MemData, self::RuntimeError> {
        self.jobs[0].call(id)
    }

}
