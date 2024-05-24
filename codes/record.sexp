(type t : (record
    (x : int)
    (y : bool)
))
(let a : t (record
    (x : 1)
    (y : true)
))
(let x2 : ([] t :x) ([] a :x))
