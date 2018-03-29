// #![recursion_limit="128"]

mod vm;

/// (OpCode ident n val typ [mute])
macro_rules! op {
    (@inst [$($done:expr),*]  $ident:ident $($rest:tt)*) => {
        op!(@ident [ $($done,)* OpCode::$ident ] $($rest)*)
    };

    (@ident [$($done:expr),*] ) => {
        op!(@ident [ $($done),* ] _ )
    };
    (@ident [$($done:expr),*] _ $($rest:tt)*) => {
        op!(@n [ $($done,)* None ] $($rest)*)
    };
    (@ident [$($done:expr),*]  $ident:ident $($rest:tt)*) => {
        op!(@n [ $($done,)* Some(___BinIdent::$ident as IdentID) ] $($rest)*)
    };

    (@n [$($done:expr),*] ) => {
        op!(@n [ $($done),* ] _ )
    };
    (@n [$($done:expr),*] _ $($rest:tt)*) => {
        op!(@val [ $($done,)* None ] $($rest)*)
    };
    (@n [$($done:expr),*]  $n:tt $($rest:tt)*) => {
        op!(@val [ $($done,)* Some($n as Quantif) ] $($rest)*)
    };

    (@val [$($done:expr),*] ) => {
        op!(@val [ $($done),* ] _ )
    };
    (@val [$($done:expr),*] _ $($rest:tt)*) => {
        op!(@typ [ $($done,)* None ] $($rest)*)
    };
    (@val [$($done:expr),*]  $val:ident $($rest:tt)*) => {
        op!(@typ [ $($done,)* Some(___BinConst::$val as ConstID) ] $($rest)*)
    };

    (@typ [$($done:expr),*] ) => {
        op!(@typ [ $($done),* ] _ )
    };
    (@typ [$($done:expr),*] _ $($rest:tt)*) => {
        op!(@mute [ $($done,)* None ] $($rest)*)
    };
    (@typ [$($done:expr),*]  $typ:ident $($rest:tt)*) => {
        op!(@mute [ $($done,)* Some(Type::$typ) ] $($rest)*)
    };

    (@mute [$($done:expr),*] ) => {
        op!(@end $($done,)* false )
    };
    (@mute [$($done:expr),*] & ) => {
        op!(@end $($done,)* true )
    };

    (@end $($args:expr),*) => {
        Op::new($($args),*)
    };

    ($($rest:tt)*) => {
        op!(@inst [] $($rest)*)
    }
}

macro_rules! instructions {
    {@mid [$($done:expr),*] ($($tts:tt)*) $($rest:tt)*} => {
        instructions!{@mid [ $($done,)* op!($($tts)*) ] $($rest)*}
    };
    {@mid [$($done:expr),*]} => {
        vec![$($done),*]
    };
    {$($rest:tt)*} => {
        instructions!{@mid [] $($rest)* }
    };
}

macro_rules! consts {
    {@const $id:ident [$($ids:ident),*] [$($vals:expr),*] ($ident:ident = $($val:tt)*) $($rest:tt)*} => {
        consts!{@const $id [$($ids,)* $ident] [$($vals,)* MemData::$($val)*] $($rest)* }
    };

    {@const $id:ident [$($ids:ident),*] [$($vals:expr),*] } => {
        #[allow(non_camel_case_types)]
        #[repr(u16)]
        enum ___BinConst { $($ids),* }

        let $id = vec![$($vals),*];
    };

    { $ident:ident = $($rest:tt)*} => {
        consts!{@const $ident [] [] $($rest)* }
    };

}

macro_rules! idents {
    { $($idents:ident),* } => {
        #[allow(non_camel_case_types)]
        #[repr(u16)]
        enum ___BinIdent { $($idents),* }
    }
}


macro_rules! program {
    { {$($idents:tt)*} {$($consts:tt)*} {$($instructions:tt)*} } => {
        {
            idents!{ $($idents)* };
            consts!{ ___bin_consts = $($consts)* };

            Bin::new(
                instructions!{ $($instructions)* }.into(),
                ___bin_consts
                )
        }
    }
}

use vm::{
    Op,
    OpCode,
    Type,
    Instructions,
    Bin,
    IdentID,
    ConstID,
    Quantif,
    MemData,
    // ConstData,
};

fn main() {
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
                        (a = Int(10))
                        (b = Str("abc".to_owned()))
                    }
                    {
                        (PSS)
                        (DVR a _ a _ &)
                        (DVR b _ b _ &)
                        (LVR b)
                        // (LVR a)
                        // (CNV _ 1 _ Str)
                        // (CAT _ 2)
                        (DSP)
                        (PPS)
                    }
        });

    let _ = lisp.call(&id);
}

