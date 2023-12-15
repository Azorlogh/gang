use std::f32::consts::TAU;

use gang::g3::*;

fn main() {
	let myvector = V1::new(1.0, 2.0, 3.0);
	let myrotor = Rot::from_v2_angle(V2::E01, TAU / 4.0);
	println!("{:.2?}", myrotor.rotate(myvector));
}
