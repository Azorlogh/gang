# Gang

Toy crate for geometric algebra geared towards game development.

It allows creating typed N-dimensional VGAs (Clifford algebras with signature Cl(N, 0, 0)) using macros.

It is barely started, most operations are unimplemented. The code is also ugly.

Currently implements VGA only but could be expanded in theory.

## Why?

Why not? Also, to make 4d games. And because geometric algebra is very elegant, even in 2d/3d.

## Implementation status

- [x] Addition/Subtraction
- [x] Scalar multiplication
- [x] Rotor multiplication
- [x] Rotor::rotate
- [ ] VK multiplication
- [ ] Wedge product
- [ ] Left-contraction
- [ ] Right contraction
- [ ] Scalar product
- [ ] Fat dot product
- [ ] ???

## How to use

If you want 2d or 3d VGA, you can use `gang::g2` or `gang::g3` (using features `dim2` and `dim3` respectively).

The axes are numbered from starting from 0: e0, e1, e2...

There are two kinds of types currently in these modules:

`VK` (e.g. `V0`, `V1`, `V2`...), are K-Vectors (multivectors with only the K-grade elements).  
Thus `V0` are just scalars, `V1` are vectors, `V2` are bivectors, etc...

The crate also defines Rotors, called `Rot`.

Example usage
```rust
use gang::g3::*;
use std::f32::consts::TAU;

let myvector = V1::new(1.0, 2.0, 3.0);
let myrotor = Rot::from_v2_angle(V2::E01, TAU / 4.0);
println!("{:.2?}", myrotor.rotate(myvector));
V1 { e0: -2.00, e1: 1.00, e2: 3.00 }
```

This macro can be used to make arbitrary VGAs: `gang_macros::gang!(N);`

