(let x 3)
(case
    ((== x 1) => (dbg 1))
    (true => (dbg 2))
)