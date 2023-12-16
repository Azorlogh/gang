pub trait Rotate<Rhs = Self> {
	type Output;

	fn rotate(self, rhs: Rhs) -> Self::Output;
}

pub trait Wedge<Rhs = Self> {
	type Output;

	fn wedge(self, rhs: Rhs) -> Self::Output;
}

pub use gang_macros::gang;

#[cfg(feature = "g2")]
pub mod g2 {
	use crate as gang;
	pub use crate::prelude::*;
	gang_macros::gang!(2);

	#[cfg(feature = "mint")]
	mod mint_impl {
		use super::*;
		impl From<V1> for mint::Vector2<f64> {
			fn from(v: V1) -> Self {
				Self {
					x: v.e0 as f64,
					y: v.e1 as f64,
				}
			}
		}
		impl From<mint::Vector2<f64>> for V1 {
			fn from(v: mint::Vector2<f64>) -> Self {
				Self {
					e0: v.x as f32,
					e1: v.y as f32,
				}
			}
		}
	}
}

#[cfg(feature = "g3")]
pub mod g3 {
	use crate as gang;
	pub use crate::prelude::*;
	gang_macros::gang!(3);
}

#[cfg(feature = "g4")]
pub mod g4 {
	use crate as gang;
	pub use crate::prelude::*;
	gang_macros::gang!(4);
}

mod prelude {
	pub use super::{Rotate, Wedge};
}
