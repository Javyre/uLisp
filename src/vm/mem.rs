use std::collections::HashMap;
use std::cell::{
    RefCell,
    Ref,
    RefMut,
};
use std::rc::{
    Rc,
    Weak,
};

use super::{
    // ConstData,
    MemData,
    // Instructions,
    IdentID,
    ConstID,
    Error,
};

pub type Constants = Vec<Rc<MemData>>;
pub type VarStrings = HashMap<String, IdentID>;

#[derive(Clone)]
pub struct Environment {
    // frames: Vec<Rc<RefCell<Frame>>>,
    env_head: Rc<RefCell<EnvNode>>,
    env_tail: Rc<RefCell<EnvNode>>,
    len: usize,

    consts:      Rc<RefCell<Constants>>,
    var_strings: Rc<RefCell<VarStrings>>
}

struct EnvNode {
    parent: Option<Rc<RefCell<EnvNode>>>,
    child:  Option<Weak<RefCell<EnvNode>>>,
    frame:  Frame,
}

pub struct Frame {
    vars: HashMap<IdentID, Rc<MemData>>,
}

// pub struct Memory {
//     var_strings: HashMap<IdentID, String>,
//     consts: Vec<MemData>,
//     // stack: Vec<Frame>,
//     // detached_envs: Vec<Environment>,
//     stack: Environment,
//     // scope: usize,
// }

impl Environment {
    pub fn new(consts: Rc<RefCell<Constants>>) -> Self {
        let node = Rc::new(RefCell::new(EnvNode::new()));
        Self {
            // frames: vec![Rc::new(RefCell::new(Frame::new()))],
            env_head: Rc::clone(&node),
            env_tail: node,
            len: 1,

            consts,
            var_strings: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn append(&mut self, other: Environment) {
        self.len += other.len;

        let old_tail = ::std::mem::replace(&mut self.env_tail, other.env_tail);

        other.env_head.borrow_mut().set_parent(Rc::clone(&old_tail));
        old_tail.borrow_mut().set_child(other.env_head);
    }

    pub fn define(&mut self, ident: IdentID, val: MemData) -> Result<(), Error> {
        Ok(self.env_tail.borrow_mut().define(ident, val))
    }

    fn define_n(&mut self, frame: usize, ident: IdentID, val: MemData) -> Result<(), Error> {
        if frame >= self.len {
            Err(Error::BadScopeIndex(frame))
        } else {
            Ok(self.env_head.borrow_mut().define_n(frame, ident, val))
        }
    }

    pub fn get(&self, ident: &IdentID) -> Result<MemData, Error> {
        self.env_tail.borrow().get(ident)
        // Ref::map(self.env_tail.borrow(), |t| t.get(ident))
    }

    // pub fn get_node_mut(&self, i: usize) -> Result<Rc<RefCell<EnvNode>>, Error> {
    //     if i >= self.len {
    //         Err(Error::BadScopeIndex(i))
    //     } else {
    //         Ok(self.env_head.borrow().get_child_n(i).unwrap())
    //     }
    // }

    pub fn new_frame(&mut self) {
        self.len += 1;

        let node = Rc::new(RefCell::new(EnvNode::new()));
        let old_tail = ::std::mem::replace(&mut self.env_tail, node);
        old_tail.borrow_mut().set_child(Rc::clone(&self.env_tail));
        self.env_tail.borrow_mut().set_parent(old_tail);
    }

    pub fn pop_frame(&mut self) -> Result<(), Error> {
        self.len -= 1;

        let new_tail = self.env_tail.borrow_mut().pop_parent().ok_or(Error::IllegalStackPop)?;
        let _ = new_tail.borrow_mut().pop_child();
        self.env_tail = new_tail;
        Ok(())
    }

    pub fn max_id(&self) -> Option<IdentID> {
        self.env_head.borrow().max_id(self.len-1)
        // let mut n_node = Some(&self.env_head);
        // let mut max = 0 as IdentID;

        // while n_node.is_some() {
        //     let node = n_node.unwrap();
        //     max = ::std::cmp::max(max, *node.borrow().get_frame().max_id().unwrap_or(&0));

        //     n_node = node.borrow().get_child();
        // }

        // max
    }

    // pub fn new_frame(&mut self) {
    //     self.frames.push(Rc::new(RefCell::new(Frame::new())))
    // }

    // pub fn pop_frame(&mut self) -> Result<(), Error> {
    //     self.frames.pop().ok_or(Error::IllegalStackPop).map(|_|())
    // }

    // pub fn generate_ident_id(&self) -> IdentID {
    //     self.frames.iter()
    //         .map(|s| s.borrow().max_id().unwrap_or(&0) + 1)
    //         .max().unwrap_or(0)
    // }
    
    pub fn get_const(&self, constid: &ConstID) -> Result<MemData, Error> {
        self.consts
            .borrow()
            .get(*constid as usize)
            .map_or_else(|| Err(Error::ConstantNotFound(*constid)),
                         |r| Ok(MemData::Pointer(Rc::clone(r))))
    }

    pub fn new_ident_id(&mut self, var_str: Option<String>) -> IdentID {
        let id = self.max_id().unwrap_or(0) + 1;
        if let Some(s) = var_str {
            self.bind_var_string(s, id);
        }
        id
    }

    pub fn bind_var_string(&mut self, s: String, id: IdentID) {
        let _ = self.var_strings.borrow_mut().insert(s,  id);
    }

    pub fn get_ident(&self, s: &String) -> Option<IdentID> {
        self.var_strings.borrow().get(s).map(|id| *id)
    }

    pub fn load_const(&mut self, val: MemData) -> usize {
        let mut c = self.consts.borrow_mut();
        c.push(Rc::new(val));
        c.len() - 1
    }

    pub fn max_const_id(&self) -> Option<usize> {
        let len = self.consts.borrow().len();
        if len == 0 {
            None
        } else {
            Some(len - 1)
        }
    }
}

impl ::std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "Environment {{ head: {:x}, tail: {:x}, len: {} }}",
               unsafe { 
                   ::std::mem::transmute::<_, usize>(Rc::into_raw(self.env_head.clone())) 
               },
               unsafe {
                   ::std::mem::transmute::<_, usize>(Rc::into_raw(self.env_tail.clone()))
               },
               self.len,
               )
    }
}

impl ::std::cmp::PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len &&
            Rc::ptr_eq(&self.env_tail, &other.env_tail) &&
            Rc::ptr_eq(&self.env_head, &other.env_head) &&
            Rc::ptr_eq(&self.consts,   &other.consts)   && 
            Rc::ptr_eq(&self.var_strings, &other.var_strings)
    }
}

impl ::std::cmp::Eq for Environment {}


impl EnvNode {
    pub fn new() -> Self {
        Self {
            parent: None,
            child: None,
            frame: Frame::new(),
        }
    }

    fn set_parent(&mut self, parent: Rc<RefCell<EnvNode>>) {
        self.parent = Some(parent)
    }

    fn set_child(&mut self, child: Rc<RefCell<EnvNode>>) {
        self.child = Some(Rc::downgrade(&child))
    }

    fn get_parent(&self) -> Option<&Rc<RefCell<EnvNode>>> {
        self.parent.as_ref()
    }

    // fn get_child(&self) -> Option<&Rc<RefCell<EnvNode>>> {
    //     self.child.as_ref().map(|c| c.upgrade().unwrap()).as_ref()
    // }
    fn get_child(&self) -> Option<&Weak<RefCell<EnvNode>>> {
        self.child.as_ref()
    }

    fn define(&mut self, ident: IdentID, val: MemData) { 
        self.frame.define(ident, val)
    }

    fn define_n(&mut self, n: usize, ident: IdentID, val: MemData) {
        if n == 0 {
            self.define(ident, val)
        } else {
            let c = self.child.as_ref().unwrap().upgrade().unwrap();
            c.borrow_mut().define_n(n-1, ident, val);
        }
    }

    fn get(&self, ident: &IdentID) -> Result<MemData, Error> {
        let val = self.frame.get(ident);
        if val.is_some() {
            Ok(val.unwrap())
        } else {
            if self.parent.is_some() {
                self.parent.as_ref().unwrap().borrow().get(ident)
            } else {
                // FIXME: scope for error set to const 0
                Err(Error::VariableNotFound(0, *ident))
            }
        }
    }

    fn max_id(&self, depth: usize) -> Option<IdentID> {
        if depth == 0 {
            return self.frame.max_id().map(|id| *id);
        }

        let c = self.child.as_ref().map(|c| c.upgrade().unwrap()).unwrap();
        let c = c.borrow();
        ::std::cmp::max(
            self.frame.max_id().map(|id| *id),
            // self.child.as_ref().unwrap().upgrade().unwrap().borrow().max_id(depth-1)
            c.max_id(depth-1)
            )
    }

    // FIXME: this funcntion should return a reference and not perform any cloning
    fn get_parent_n(&self, n: usize) -> Option<Rc<RefCell<EnvNode>>> {
        let parent = self.get_parent();
        if n == 0 { return parent.map(|p| p.clone()); }
        let parent = parent.unwrap().borrow();

        parent.get_parent_n(n-1)
    }

    // FIXME: this funcntion should return a reference and not perform any cloning
    fn get_child_n(&self, n: usize) -> Option<Rc<RefCell<EnvNode>>> {
        let child = self.get_child();
        // if n == 0 { return child.map(|c| c.clone()); }
        if n == 0 { return child.map(|c| c.upgrade().unwrap()); }
        let child = child.unwrap().upgrade().unwrap();
        let child = child.borrow();

        child.get_child_n(n-1)
    }

    // fn get_parent(&self) -> Option<Rc<RefCell<EnvNode>>> {
    //     self.parent.as_ref().map(|ref p| Rc::clone(p))
    // }

    // fn get_child(&self) -> Option<Rc<RefCell<EnvNode>>> {
    //     self.child.as_ref().map(|ref c| Rc::clone(c))
    // }

    fn pop_parent(&mut self) -> Option<Rc<RefCell<EnvNode>>> {
        ::std::mem::replace(&mut self.parent, None)
    }

    fn pop_child(&mut self) -> Option<Rc<RefCell<EnvNode>>> {
        ::std::mem::replace(&mut self.child, None).map(|c| c.upgrade().unwrap())
    }

    fn get_frame(&self) -> &Frame {
        &self.frame
    }

    fn get_frame_mut(&mut self) -> &mut Frame {
        &mut self.frame
    }
}

impl Frame {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn define(&mut self, id: IdentID, val: MemData) {
        self.vars.insert(id, Rc::new(val));
    }

    pub fn get(&self, id: &IdentID) -> Option<MemData> {
        self.vars.get(id).map(|rc| MemData::Pointer(Rc::clone(rc)))
    }

    pub fn max_id(&self) -> Option<&IdentID> {
        self.vars.keys().max()
    }
}

// impl Memory {
//     pub fn new() -> Self {
//         let consts = Rc::new(RefCell::new(Vec::new()));
//         Self {
//             var_strings: HashMap::new(),
//             consts: vec![], // NOTE: maybe use Option here?
//             // stack: vec![Frame::new()],
//             stack: Environment::new(consts),
//         }
//     }

//     #[inline]
//     pub fn push_scope(&mut self) { self.stack.new_frame() }

//     #[inline]
//     pub fn pop_scope(&mut self) -> Result<(), Error> {
//         self.stack.pop_frame()
//     }

//     pub fn load_consts(&mut self, mut consts: Vec<MemData>) -> usize {
//         let r = self.consts.len();
//         self.consts.append(&mut consts);
//         r
//     }

//     pub fn bind_idents(
//         &mut self, mut idents: Vec<IdentID>, mut var_strings: HashMap<IdentID, String>)
//         -> (Vec<IdentID>, HashMap<IdentID, String>) {

//             let mut new_var_strings = HashMap::new();
//             idents.iter_mut().for_each(|i| {
//                 let ni = self.bind_ident_id(var_strings.get(i).map(|s| s.to_owned()));
//                 if var_strings.contains_key(i) {
//                     new_var_strings.insert(ni, var_strings.remove(i).unwrap());
//                 }
//                 *i = ni;
//             });

//             (idents, new_var_strings)
//     }

//     // All IdentID creation for memory should pass by here
//     // doing otherwise will cause spooky behaviour
//     #[inline]
//     pub fn bind_ident_id(&mut self, var_str: Option<String>) -> IdentID {
//         // let id = self.stack.generate_ident_id();
//         let id = self.stack.max_id().unwrap_or(0) + 1;
//         if let Some(s) = var_str {
//             self.var_strings.insert(id, s);
//         }
//         id
//     }

//     // #[inline]
//     // pub fn define(&mut self, scope: usize, ident: IdentID, val: MemData) -> Result<(), Error> {
//     //     // Ok(self.stack.get_frame_mut(scope)?.define(ident, val))
//     //     self.stack.define(scope, ident, val)
//     // }
//     pub fn define(&mut self, ident: IdentID, val: MemData) -> Result<(), Error> {
//         self.stack.define(ident, val)
//     }

//     pub fn get(&self, ident: &IdentID) -> Result<MemData, Error> {
//         self.stack.get(ident)
//     }
//     // pub fn get<'a>(&self, scope: usize, ident: &IdentID) -> Result<Ref<MemData>, Error> {
//     // // pub fn get<'a>(&self, scope: usize, ident: &IdentID) -> Result<&MemData, Error> {
//     //     for scope in (0..scope+1).rev() {

//     //         // let f = self.stack.get_frame(scope)?;
//     //         let f = Ref::map(self.stack.get_node(scope)?, |n| n.get_frame());
//     //         if f.get(ident).is_some() {
//     //             return Ok(Ref::map(f, |f| f.get(ident).unwrap()));
//     //             // return Ok(f.get(ident).unwrap());
//     //         }
//     //     }

//     //     Err(Error::VariableNotFound(scope, *ident))
//     // }

//     pub fn get_const(&self, constid: &ConstID) -> Result<&MemData, Error> {
//         self.consts
//             .get(*constid as usize)
//             .map_or_else(|| Err(Error::ConstantNotFound(*constid)),
//                          |r| Ok(r))
//     }
// }
