# Gang (⚠ very incomplete)

Toy crate for vector geometric algebras geared towards game development.

Has strongly-typed VGAs from dimensions 2 to 5.

The types and methods are generated from a proc macro, which could work for more dimensions in theory, but the combinatorial nature of higher dimensions make this impractical.

## Why?

Because geometric algebra is elegant, and makes it easier to work in higher dimensions.
Also, why not? 😄

## Implementation status

- [x] Addition/Subtraction
- [x] Scalar multiplication
- [x] Rotor::rotate
- [ ] Geometric product
  - [x] For rotors
  - [ ] For V1
  - [ ] ??
- [x] Wedge product
- [ ] Left-contraction
- [ ] Right contraction
- [ ] Scalar product
- [ ] Fat dot product
- [ ] ???

## How to use

Enable features `g2`, `g3`, `g4` or `g5`, and use the types in `gang::g2`, `gang::g3`, etc.

The axes are numbered starting from 0: `e0`, `e1`, `e2`...

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
// V1 { e0: -2.00, e1: 1.00, e2: 3.00 }
```

