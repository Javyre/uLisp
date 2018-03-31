use std::collections::HashMap;

use super::{
    // ConstData,
    MemData,
    // Instructions,
    IdentID,
    ConstID,
    Error,
};

pub struct Memory {
    consts: Vec<MemData>,
    data: Vec<HashMap<IdentID, MemData>>,
    // scope: usize,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            consts: vec![], // NOTE: maybe use Option here?
            data: vec![HashMap::new()],
            // scope: 0,
        }
    }

    pub fn push_scope(&mut self) {
        self.data.push(HashMap::new())
    }

    pub fn pop_scope(&mut self) -> Result<(), Error> {
        self.data.pop().map_or_else(|| Err(Error::IllegalStackPop), |_| Ok(()))
    }

    pub fn load_consts(&mut self, mut consts: Vec<MemData>) -> usize {
        let r = self.consts.len();
        self.consts.append(&mut consts);
        r
    }

    pub fn generate_ident_id(&self, scope: usize) -> IdentID {
        self.data[scope].keys().max().unwrap_or(&0) + 1
    }

    pub fn define(&mut self, scope: usize, ident: IdentID, val: MemData) {
        // self.data[self.scope].insert(ident, val);
        self.data[scope].insert(ident, val);
    }

    pub fn get(&self, scope: usize, ident: &IdentID) -> Result<&MemData, Error> {
        let mut r: Option<&MemData> = None;
        for scope in (0..scope+1).rev() {
            r = self.data[scope].get(ident);
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

    // pub fn inc_scope(&mut self, n: usize) { self.scope += n; }
    // pub fn dec_scope(&mut self, n: usize) { self.scope -= n; }
}
