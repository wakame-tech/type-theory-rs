(let + any (lam (a : int) (lam (b : int) (add! a b))))
(let p1 any (+ 1))
(p1 2)