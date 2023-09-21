pub trait Rotate<Rhs = Self> {
	type Output;

	fn rotate(self, rhs: Rhs) -> Self::Output;
}

gang_macros::gang!(2);
fn main() {}
