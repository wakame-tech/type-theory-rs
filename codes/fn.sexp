(include std/prelude.sexp)

(let g (fn x (fn y (+ x y))))
((g 1) ((g 2) 3))
