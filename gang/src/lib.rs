pub trait Rotate<Rhs = Self> {
	type Output;

	fn rotate(self, rhs: Rhs) -> Self::Output;
}

pub use gang_macros::gang;

mod g3 {
	use crate as gang;
	pub use crate::Rotate;
	gang_macros::gang!(3);
}

#[cfg(test)]
mod test {
	use std::f32::consts::PI;

	use crate::g3::*;

	#[test]
	fn test() {
		let a = V1::new(1.0, 2.0, 3.0);
		let r = Rot::from_v2_angle(V2::E01, PI / 2.0);
		assert!(r.rotate(a).abs_diff_eq(V1::new(2.0, -1.0, 3.0), 1e-6));
	}
}
