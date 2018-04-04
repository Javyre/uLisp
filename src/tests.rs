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

#[test]
fn prg2() {
    let mut lisp: vm::VM = vm::VM::new();
    let id = lisp.load(
        program! {
                    { cdar, l }
                    {
                        (#a = Int(8))
                        (#b = Int(1))
                        (#c = Int(2))
                        (#d = Int(3))
                    }
                    {
                        (REC (3))
                            (DVR l &)
                            (CDR l)
                            (CAR (1))
                        // (DFN (3) cdar)
                        (LMB (3))
                        (DVR cdar &)

                        // -------- //

                        (LVR #a)
                                    (LVR #b)
                                        (LVR #c)
                                        (LVR #d)
                                    (CNS)
                                (CNS)
                            (CLL cdar)
                        (ADD (2)) // #a + #c
                    }
        });

    assert_eq!(lisp.call(&id).unwrap(), MemData::Int(10));
}


#[test]
fn prg3() {
    let mut lisp: vm::VM = vm::VM::new();
    let id = lisp.load(
        program! {
                    { }
                    {
                        (#t = Bool(true))
                        (#strue = Str("true\n".to_owned()))
                        (#sntrue = Str("not true\n".to_owned()))
                        (#a = Int(123))
                        (#b = Int(321))
                        (#_one = Int(1))
                        (#_two = Int(2))
                    }
                    {
                            (REC (5))
                                (PSS)
                                        (LVR #sntrue)
                                    (DSP &)
                                    (LVR #a)
                                (PPS)
                            (LMB (5))
                            (REC (9))
                                // BEGIN INNER IF
                                    (REC (5))
                                        (PSS)
                                                (LVR #strue)
                                            (DSP &)
                                            (LVR #b)
                                        (PPS)
                                    (LMB (5))
                                    (LVR #t)
                                (IFT)
                                // END INNER IF
                            (LMB (9))
                                (LVR #_one)
                                (LVR #_two)
                            (CGT (2)) // greater than on 2 laxt vals
                        (IFE)
                    }
        });

    assert_eq!(lisp.call(&id).unwrap(), MemData::Int(321));
}
