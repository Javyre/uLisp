# Code:

    (let ((a 10) (b "abc"))
        (display (concat b (int->str a))))

    (define (cadr l)
        (car (cdr l)))

    (+ 8 (cdar (1 . (2 . 3)))) ;; should be = 10

    ;; (- 7 (cdar '(4 5 6)))
    ;; (+ 8 (cdar '(1 2 3)))

# IR:

    (do
        (define-var a 10)
        (define-var b "abc")
        (display (concat b
                         (int->str a))))

    (define-fun cdar l (cdr (car l)))

    (+ 8 (call-fun cdar (cons 1 (cons 2 3))))

# BC:

    PSS
        DVR 'a' #Int(10)
        DVR 'b' #Str('abc')

            LVR 'b'
            LVR 'a'
            CNV #1 <int> <str>
            PRA #2
            CAT #2 ;; concat last 2 in A
            PRA #1 ;; push last R to [] in A

        DSP   ;; display consuming last 1 item from A
              ;; keep in mind all "functions" return pushing to R even if empty return
        PRA #1 ;; could also just do a PLA
        RRR #2 ;; get rid of the returns from the DVRs

    PAT 0   ;; pop from A to tmp id:0
    LTR 0   ;; load tmp id:0 to R
    PPS

    ;; ---
    ;; NO A REGISTER!!

    PSS
    DVR 'a' #Int(10)
    DVR 'b' #Str('abc')
    RRR 2
    LVR 'b'
    LVR 'a'
    CNV 1 <str>
    CAT 2
    DSP
    PPS



    ;; ---


    REC #10

    PSS
    ;; PAT #1 #0
    PAV 'l' ;; Pop from A to load to 'l' variable
            LVR 'l'
            PLA ;; == PRA #1
        CAR
        PLA
    CDR
    PPS

    PRA #10
    DFN 'cadr'

    ;; --
    ;; NO A REGISTER!!


    (do
        (define a '(1 2 3 4))
        (set-item a 2 5))

    REC 3
        DVR 'l' mute ;; load parameter(s) to scope
        CDR 'l'
        CAR 1
    LMB 3
    > DFN 3 'cdar'
    DVR 'cdar' mute

    ;; --

        LVR #8
                LVR #Int(1)
                    LVR #Int(2)
                    LVR #Int(3)
                CNS
            CNS
        CLL 'cdar'
    ADD 2




## OPS:

32bit words
opcode | u18/ident | type/const | type


### Bin Layout:
header:
    - version info
    - sizeof(MemData)
    - size of operations section
    - size of data section
    - terminator sequence

operation:
    - read as [T] where T: Op

data:
    - read as [T] where T: MemData

Execution begines at first element of operations array


### OPERATION:

    [6bit OP][18bit n/ident][8bit flags] + optional[multi]
    .------------------------^^^^^^^^^^
    |
    V
    flags:

    [5bit ---][1bit mute][1bit n/ident flag][1bit expansion flag]

### A and R regs
RRR n    : pop last <n> vals from R
            `[6bit OP][18bit n][8bit ---]`

### Othters

> alias BAG -> PNA
// alias EFN n -> PRA n

PSS             : push stack frame to allow later
                   restoration of parent scope variables
                   and tmp values
                   `[6bit OP][26bit ---]`
PPS             : exit the scope
                   `[6bit OP][26bit ---]`

DFN n ident     : define function with name <ident> to current scope with last <n>
                   elements of register as body
                   `[6bit OP][18bit n][8bit ---] + [18bit ident][14bit ---]`

DVR ident [val] : store var name <ident> into the scope's memory with const
                   val or val from register
                   `[6bit OP][18bit ident][7bit ---][1bit reg/const flag] + [32bit const]`
LVR ident|val   : load val from <ident> in scope or constant <val> to R register
                   `[6bit OP][18bit ident][7bit ---][1bit reg/const flag] + [32bit const]`

CLL ident|n     : call a function <ident> in scope or the last n values in register as a function
                   `[6bit OP][18bit ident/n][7bit ---][1bit var/reg flag]`

CNV ident|n typ : convert value of <ident> to type <typ>
                   `[6bit OP][18bit ident/n][7bit ---][1bit var/reg flag] + [16bit type][16bit ---]`

CAT n           : concat a number of str values
                   `[6bit OP][18bit n][8bit ---]`
CNS val|ident   : construct a pair of <val>/(val of <ident>) and 1 value popped from R
                   `[6bit OP][18bit ident][7bit ---][1bit const/ident flag] + [32bit const]`

CAR ident|n     : get the car of <ident> in scope or of <n> elements in register
                   `[6bit OP][18bit ident/n][7bit ---][1bit var/reg flag]`

CDR ident|n     : get the cdr of <ident> in scope or of <n> elements in register
                   `[6bit OP][18bit ident/n][7bit ---][1bit var/reg flag]`

ADD ident|n     : add last in R with val of <ident> or add up last <n> vals in R
                   `[6bit OP][18bit ident/n][7bit ---][1bit var/reg flag]`

SUB ident|n     : subtract last in R by val of <ident> or subtract last <n> vals in R
                   (one by one in order of oldest->newest)
                   `[6bit OP][18bit ident/n][7bit ---][1bit var/reg flag]`

MUL ident|n     : multiply last in R with val of <ident> or multiply last <n> vals in R
                   `[6bit OP][18bit ident/n][7bit ---][1bit var/reg flag]`

DIV ident|n     : divide last in R by val of <ident> or divide last <n> vals in R
                    (one by one in orer of oldest->newest)
                    `[6bit OP][18bit ident/n][7bit ---][1bit var/reg flag]`

DSP             : print an str
                 `[6bit OP][26bit ---]`


# Conditionals:

    (cond ((> 1 2)
           (display "not true")
           123)
          (#t
           (display "true")
           321))

    ;; -----


    (if (> 1 2)
        (do
            (display "not true")
            123)
        (if #t
            (do
                (display "true")
                321)))

    ;; -----
    ;; IFT: if with only one conditionnally evaluated blcok
    ;; IFE: if with an else block as well


        REC 5
            PSS
                    LVR #Str("not true")
                DSP &
                LVR #Int(123)
            PPS
        LMB 5
        REC 9
            ;; BEGIN INNER IF
                REC 5
                    PSS
                            LVR #Str("true")
                        DSP &
                        LVR #Int(321)
                    PPS
                LMB 5
                LVR #Bool(true)
            IFT
            ;; END INNER IF
        LMB 9
            LVR #Int(1)
            LVR #Int(2)
        CGT 2 ;; greater than on 2 laxt vals
    IFE

    ;; -------------
    ;; closures

        (do
            (define foo
                (let ((x 4))
                    (lambda (y) (+ x y))))
            (foo 6)) ;; == 10

    ;; ---

        (do
            (define foo
                (do
                    (define-var x 4)
                    (lambda (y) (+ x y))))
            (foo 6)) ;; == 10

    ;; ---

        PSS
            PSS
                    LVR #Int(4)
                DVR x &
                    REC 4
                        DVR y &
                            LVR x ;; we need to capture x
                            LVR y
                        ADD 2
                    PRC 4
                    CAP x ;; capture x (does not return anything)
                    ;; ENV 1
                LMB
            PPS
            DVR foo &
                LVR #Int(6)
            CLL foo
        PPS




