(let id : ((a) -> a) (external id))
(let + : ((int int) -> int) (external +))
(let - : ((int int) -> int) (external -))
(let % : ((int int) -> int) (external %))
(let not : ((bool) -> bool) (external not))
(let & : ((bool bool) -> bool) (external &))
(let | : ((bool bool) -> bool) (external |))
(let == : ((int int) -> bool) (external ==))
(let != : ((int int) -> bool) (external !=))
(let [] : ((a b) -> ([] a b)) (external []))
(let map : ((((a) -> b) (vec a)) -> (vec b)) (external map))
(let filter : ((((a) -> bool) (vec a)) -> (vec a)) (external filter))
(let range : ((int int) -> (vec int)) (external range))
(let dbg : ((a) -> a) (external dbg))
(let to_string : ((a) -> str) (external to_string))
