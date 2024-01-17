# draw-dag

```rust
let a = allocate(1, Vec::new());
let b = allocate(2, Vec::new());
let c = allocate(3, Vec::new());
let d = allocate(4, Vec::new());
let e = allocate(5, vec![a]);
let f = allocate(6, Vec::new());
let g = allocate(7, vec![b]);
let h = allocate(8, vec![d, c]);
let i = allocate(9, vec![e]);
let j = allocate(10, vec![h, g, f]);
let k = allocate(11, vec![j, i]);
let l = allocate(12, vec![k]);

let graph = draw_dag(l, 1);
```

becomes

```text
12
│
11
├───┐
9   10
│   ├──┬──┐
5   6  7  8
│      │  ├──┐
1      2  3  4
```