use vm;
use vm::*;

#[test]
fn prg0() {
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

    assert_eq!(lisp.call(&id).unwrap(), MemData::Nil);
}

#[test]
fn prg1() {
    let mut lisp: vm::VM = vm::VM::new();
    let id = lisp.load(
        program! {
                    { }
                    { (#a = Int(9) )}
                    { (LVR #a) }
        });

    assert_eq!(lisp.call(&id).unwrap(), MemData::Int(9))
}