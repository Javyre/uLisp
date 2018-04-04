use std::collections::HashMap;

use super::{
    // ConstData,
    MemData,
    // Instructions,
    IdentID,
    ConstID,
    Error,
};

pub struct Stack {
    frames: Vec<Environment>,
}

pub struct Environment {
    vars: HashMap<IdentID, MemData>,
}

pub struct Memory {
    consts: Vec<MemData>,
    // stack: Vec<Environment>,
    stack: Stack,
    // scope: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            frames: vec![Environment::new()],
        }
    }

    pub fn get_frame(&self, i: usize) -> Result<&Environment, Error> {
        self.frames.get(i).ok_or(Error::BadScopeIndex(i))
    }

    pub fn get_frame_mut(&mut self, i: usize) -> Result<&mut Environment, Error> {
        self.frames.get_mut(i).ok_or(Error::BadScopeIndex(i))
    }

    pub fn new_frame(&mut self) {
        self.frames.push(Environment::new())
    }

    pub fn pop_frame(&mut self) -> Result<(), Error> {
        self.frames.pop().ok_or(Error::IllegalStackPop).map(|_|())
    }

    pub fn generate_ident_id(&self) -> IdentID {
        self.frames.iter()
            .map(|s| s.max_id().unwrap_or(&0) + 1)
            .max().unwrap_or(0)
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn define(&mut self, id: IdentID, val: MemData) {
        self.vars.insert(id, val);
    }

    pub fn get(&self, id: &IdentID) -> Option<&MemData> {
        self.vars.get(id)
    }

    pub fn max_id(&self) -> Option<&IdentID> {
        self.vars.keys().max()
    }
}

impl Memory {
    pub fn new() -> Self {
        Self {
            consts: vec![], // NOTE: maybe use Option here?
            // stack: vec![Environment::new()],
            stack: Stack::new(),
        }
    }

    #[inline]
    pub fn push_scope(&mut self) { self.stack.new_frame() }

    #[inline]
    pub fn pop_scope(&mut self) -> Result<(), Error> {
        self.stack.pop_frame()
    }

    pub fn load_consts(&mut self, mut consts: Vec<MemData>) -> usize {
        let r = self.consts.len();
        self.consts.append(&mut consts);
        r
    }

    #[inline]
    pub fn generate_ident_id(&self) -> IdentID {
        self.stack.generate_ident_id()
    }

    pub fn define(&mut self, scope: usize, ident: IdentID, val: MemData) -> Result<(), Error> {
        Ok(self.stack.get_frame_mut(scope)?.define(ident, val))
    }

    pub fn get(&self, scope: usize, ident: &IdentID) -> Result<&MemData, Error> {
        let mut r: Option<&MemData> = None;
        for scope in (0..scope+1).rev() {
            r = self.stack.get_frame(scope)?.get(ident);
            if r.is_some() { break; }
        }

        r.map_or_else(|| Err(Error::VariableNotFound(scope, *ident)), |r| Ok(r))
    }

    pub fn get_const(&self, constid: &ConstID) -> Result<&MemData, Error> {
        self.consts
            .get(*constid as usize)
            .map_or_else(|| Err(Error::ConstantNotFound(*constid)),
                         |r| Ok(r))
    }
}
