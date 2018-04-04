// #![recursion_limit="128"]
#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
mod vm;

#[cfg(test)]
mod tests;


#[allow(unused_imports)]
use vm::{
    Op,
    OpCode,
    Type,
    // Instructions,
    Bin,
    IdentID,
    ConstID,
    Quantif,
    MemData,
    // ConstData,
};

fn main() {
    env_logger::init();

    // TODO: implement lisp!
    let mut lisp: vm::VM = vm::VM::new();

    /*
     *
     *  PSS
     *  DVR 'a' #Int(10)
     *  DVR 'b' #Str('abc')
     *  RRR #2
     *  LVR 'b'
     *  LVR 'a'
     *  CNV #1 <int> <str>
     *  CAT #2
     *  DSP
     *  PPS
     *
     * */

    // (OpCode ident n val typ [mute])

    let id = lisp.load(
        program! {
                    { a, b }
                    {
                        (#a = Int(10))
                        (#b = Str("abc".to_owned()))
                        (#nl = Str("\n".to_owned()))
                    }
                    {
                        (PSS)
                        (DVR a #a &)
                        (DVR b #b &)
                        (LVR b)
                        (LVR a)
                        (CNV (1) :Str)
                        (LVR #nl)
                        (CAT (3))
                        (DSP)
                        (PPS)
                    }
        });

    let _ = lisp.call(&id).unwrap();
}

