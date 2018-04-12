use std::collections::HashMap;
use std::cell::{
    RefCell,
    Ref,
    RefMut,
};
use std::rc::Rc;

use super::{
    // ConstData,
    MemData,
    // Instructions,
    IdentID,
    ConstID,
    Error,
};

pub struct Stack {
    frames: Vec<Rc<RefCell<Environment>>>,
}

pub struct Environment {
    vars: HashMap<IdentID, MemData>,
}

pub struct Memory {
    var_strings: HashMap<IdentID, String>,
    consts: Vec<MemData>,
    // stack: Vec<Environment>,
    stack: Stack,
    // scope: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            frames: vec![Rc::new(RefCell::new(Environment::new()))],
        }
    }

    pub fn get_frame(&self, i: usize) -> Result<Ref<Environment>, Error> {
        self.frames.get(i).ok_or(Error::BadScopeIndex(i)).map(|f| f.borrow())
    }

    pub fn get_frame_mut(&mut self, i: usize) -> Result<RefMut<Environment>, Error> {
        self.frames.get_mut(i).ok_or(Error::BadScopeIndex(i)).map(|f| f.borrow_mut())
    }

    pub fn new_frame(&mut self) {
        self.frames.push(Rc::new(RefCell::new(Environment::new())))
    }

    pub fn pop_frame(&mut self) -> Result<(), Error> {
        self.frames.pop().ok_or(Error::IllegalStackPop).map(|_|())
    }

    pub fn generate_ident_id(&self) -> IdentID {
        self.frames.iter()
            .map(|s| s.borrow().max_id().unwrap_or(&0) + 1)
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
            var_strings: HashMap::new(),
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

    pub fn bind_idents(
        &mut self, mut idents: Vec<IdentID>, mut var_strings: HashMap<IdentID, String>)
        -> (Vec<IdentID>, HashMap<IdentID, String>) {

            let mut new_var_strings = HashMap::new();
            idents.iter_mut().for_each(|i| {
                let ni = self.bind_ident_id(var_strings.get(i).map(|s| s.to_owned()));
                if var_strings.contains_key(i) {
                    new_var_strings.insert(ni, var_strings.remove(i).unwrap());
                }
                *i = ni;
            });

            (idents, new_var_strings)
    }

    // All IdentID creation for memory should pass by here
    // doing otherwise will cause spooky behaviour
    #[inline]
    pub fn bind_ident_id(&mut self, var_str: Option<String>) -> IdentID {
        let id = self.stack.generate_ident_id();
        if let Some(s) = var_str {
            self.var_strings.insert(id, s);
        }
        id
    }

    #[inline]
    pub fn define(&mut self, scope: usize, ident: IdentID, val: MemData) -> Result<(), Error> {
        Ok(self.stack.get_frame_mut(scope)?.define(ident, val))
    }

    pub fn get<'a>(&self, scope: usize, ident: &IdentID) -> Result<Ref<MemData>, Error> {
        for scope in (0..scope+1).rev() {

            let f = self.stack.get_frame(scope)?;
            if f.get(ident).is_some() {
                return Ok(Ref::map(f, |f| f.get(ident).unwrap()));
            }
        }

        Err(Error::VariableNotFound(scope, *ident))
    }

    pub fn get_const(&self, constid: &ConstID) -> Result<&MemData, Error> {
        self.consts
            .get(*constid as usize)
            .map_or_else(|| Err(Error::ConstantNotFound(*constid)),
                         |r| Ok(r))
    }
}
