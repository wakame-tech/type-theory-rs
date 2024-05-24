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
(fizzbuzz 3)