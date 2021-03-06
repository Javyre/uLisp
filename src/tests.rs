extern crate env_logger;

use vm;
use vm::*;

fn init_logger() {
    let _ = env_logger::try_init();
}

#[test]
fn prg0() {
    init_logger();

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
        },
        vm::LoadOpts::OVERRIDE_VAR_STRINGS);

    assert!(lisp.call(&id).unwrap().eq(&MemData::Nil).unwrap());
}

#[test]
fn prg1() {
    init_logger();

    let mut lisp: vm::VM = vm::VM::new();
    let id = lisp.load(
        program! {
                    { }
                    { (#a = Int(9) )}
                    { (LVR #a) }
        },
        vm::LoadOpts::OVERRIDE_VAR_STRINGS);

    assert!(lisp.call(&id).unwrap().eq(&MemData::Int(9)).unwrap())
}

#[test]
fn prg2() {
    init_logger();

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
        },
        vm::LoadOpts::OVERRIDE_VAR_STRINGS);

    assert!(lisp.call(&id).unwrap().eq(&MemData::Int(10)).unwrap());
}


#[test]
fn prg3() {
    init_logger();

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
                            // (LMB (5))
                            (PRC (5))
                            (REC (9))
                                // BEGIN INNER IF
                                    (REC (5))
                                        (PSS)
                                                (LVR #strue)
                                            (DSP &)
                                            (LVR #b)
                                        (PPS)
                                    // (LMB (5))
                                    (PRC (5))
                                    (LVR #t)
                                (IFT)
                                // END INNER IF
                            // (LMB (9))
                            (PRC (9))
                                (LVR #_one)
                                (LVR #_two)
                            (CGT (2)) // greater than on 2 laxt vals
                        (IFE)
                    }
        },
        vm::LoadOpts::OVERRIDE_VAR_STRINGS);

    assert!(lisp.call(&id).unwrap().eq(&MemData::Int(321)).unwrap());
}

#[test]
fn lambda() {
    init_logger();

    let mut lisp: vm::VM = vm::VM::new();
    let id = lisp.load( 
        program! {
            { foo, bar }
            { 
                (#a = Str("Heyheyhey".to_string()))
                (#b = Str("Yoyoyo".to_string()))

                (#R = Int(123))
            }
            { 
                /*
                 *
                 * (do
                 *     (define bar
                 *         (let (foo "HEYHEYHEY")
                 *             (lambda () 
                 *                 (do (display foo) foo))))
                 *
                 *     (define foo "yoyoyo")
                 *     (bar))
                 *
                 * */

                    (PSS)
                        (DVR foo #a &)
                        (REC (5))
                            (PSS)
                                    (LVR foo)
                                (DSP &)
                                (LVR foo)
                            (PPS)
                        (LMB (5))
                    (PPS)
                (DVR bar &)

                (DVR foo #b &)
                (CLL bar)
            }
        },
        vm::LoadOpts::OVERRIDE_VAR_STRINGS);

    assert!(lisp.call(&id).unwrap().eq(&MemData::Str("Heyheyhey".to_string())).unwrap());
}

#[test]
fn multi_bin() {
    init_logger();

    let msg = "woop woop woop".to_string();

    let mut lisp: vm::VM = vm::VM::new();
    let id = lisp.load(
        program! {
            { foo }
            { 
                (#a = Str(msg.clone()))
            }
            { 
                (DVR foo #a)
            }
        },
        vm::LoadOpts::OVERRIDE_VAR_STRINGS);

    let _ = lisp.call(&id).unwrap();

    let id = lisp.load(
        program! {
            { foo }
            { }
            {
                (LVR foo)
            }
        },
        vm::LoadOpts::REUSE_VAR_STRINGS);

    assert!(lisp.call(&id).unwrap().eq(&MemData::Str(msg)).unwrap());
}

