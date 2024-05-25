(let fizzbuzz
    (fn (x : int)
        (case
            ((& (== (% x 3) 0) (== (% x 5) 0)) => 'fizzbuzz')
            ((== (% x 3) 0) => 'fizz')
            ((== (% x 5) 0) => 'buzz')
            (true => 'other')
        )
    )
)
(dbg (map fizzbuzz (vec 1 2 3 4 5)))
