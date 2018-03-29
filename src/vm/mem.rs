use std::collections::HashMap;

use super::{
    // ConstData,
    MemData,
    Instructions,
    IdentID,
    ConstID,
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

    pub fn pop_scope(&mut self) {
        self.data.pop();
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

    pub fn get(&self, scope: usize, ident: &IdentID) -> Option<&MemData> {
        let mut r: Option<&MemData> = None;
        // let mut scope = self.scope + 1;
        let mut scope = scope + 1;
        while r.is_none() {
            scope -= 1;
            if scope < 0 { break; }
            r = self.data[scope].get(ident);
        }

        r
    }

    pub fn get_const(&self, constid: &ConstID) -> Option<&MemData> {
        self.consts.get(*constid as usize)
    }

    // pub fn inc_scope(&mut self, n: usize) { self.scope += n; }
    // pub fn dec_scope(&mut self, n: usize) { self.scope -= n; }
}
