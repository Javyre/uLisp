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
                        (a = Int(10))
                        (b = Str("abc".to_owned()))
                        (nl = Str("\n".to_owned()))
                    }
                    {
                        (PSS)
                        (DVR a _ a _ &)
                        (DVR b _ b _ &)
                        (LVR b)
                        (LVR a)
                        (CNV _ 1 _ Str)
                        (LVR _ _ nl)
                        (CAT _ 3)
                        (DSP)
                        (PPS)
                    }
        });

    let _ = lisp.call(&id).unwrap();
}
