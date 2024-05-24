(let f (fn (a : int) (b : int) (c : int)
    (+ (+ (dbg a) (dbg b)) c)
))
(f 1 2 3)
