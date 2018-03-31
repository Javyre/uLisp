
/// (OpCode ident n val typ [mute])
macro_rules! op {
    ($($rest:tt)*) => {
        _op!(@i $($rest)*)
    };
}

macro_rules! _op {
    (@i $inst:ident $($rest:tt)*) => {
        _op!(@a [$crate::vm::OpCode::$inst, None, None, None, None, false] $($rest)*)
    };
    (@a [$inst:expr, $a:expr, $b:expr, $c:expr, $d:expr, $e:expr] $ident:ident $($rest:tt)*) => {
        _op!(@a [$inst, Some(___BinIdent::$ident as $crate::vm::IdentID), $b, $c, $d, $e] $($rest)*)
    };
    (@a [$inst:expr, $a:expr, $b:expr, $c:expr, $d:expr, $e:expr] ($n:expr) $($rest:tt)*) => {
        _op!(@a [$inst, $a, Some($n as $crate::vm::Quantif), $c, $d, $e] $($rest)*)
    };
    (@a [$inst:expr, $a:expr, $b:expr, $c:expr, $d:expr, $e:expr] #$const:ident $($rest:tt)*) => {
        _op!(@a [$inst, $a, $b, Some(___BinConst::$const as $crate::vm::ConstID), $d, $e] $($rest)*)
    };
    (@a [$inst:expr, $a:expr, $b:expr, $c:expr, $d:expr, $e:expr] :$typ:ident $($rest:tt)*) => {
        _op!(@a [$inst, $a, $b, $c, Some($crate::vm::Type::$typ), $e] $($rest)*)
    };
    (@a [$inst:expr, $a:expr, $b:expr, $c:expr, $d:expr, $e:expr] & $($rest:tt)*) => {
        _op!(@a [$inst, $a, $b, $c, $d, true] $($rest)*)
    };
    (@a [$inst:expr, $a:expr, $b:expr, $c:expr, $d:expr, $e:expr]) => {
        Op::new($inst, $a, $b, $c, $d, $e)
    };

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
    {@const $id:ident [$($ids:ident),*] [$($vals:expr),*] (#$ident:ident = $($val:tt)*) $($rest:tt)*} => {
        consts!{@const $id [$($ids,)* $ident] [$($vals,)* MemData::$($val)*] $($rest)* }
    };

    {@const $id:ident [$($ids:ident),*] [$($vals:expr),*] } => {
        #[allow(non_camel_case_types)]
        #[repr(u16)]
        enum ___BinConst { $($ids),* }

        let $id = vec![$($vals),*];
    };

    { $ident:ident = $($rest:tt)+} => {
        consts!{@const $ident [] [] $($rest)* }
    };

    { $ident:ident = } => { };
}

macro_rules! idents {
    { $($idents:ident),+ } => {
        #[allow(non_camel_case_types)]
        #[repr(u16)]
        enum ___BinIdent { $($idents),* }
    };

    {} => {};
}


#[macro_export]
macro_rules! program {
    { {$($idents:tt)*} {$($consts:tt)*} {$($instructions:tt)*} } => {
        {
            let ___bin_consts: Vec<MemData> = vec![];
            idents!{ $($idents)* };
            consts!{ ___bin_consts = $($consts)* };

            Bin::new(
                instructions!{ $($instructions)* }.into(),
                ___bin_consts
                )
        }
    }
}
