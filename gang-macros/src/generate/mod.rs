use proc_macro2::TokenStream;
use quote::quote;

use crate::util::{basis_names, element_name, element_name_upper, Basis};

mod kvector;
mod rot;
pub(crate) use kvector::kvector_methods;
pub(crate) use rot::{impl_rotate, impl_to_matrix, rotor_methods};

pub fn constants(basis: &Basis) -> proc_macro2::TokenStream {
	let mut constants_tokens = proc_macro2::TokenStream::new();
	for base in &basis.0 {
		let element = element_name_upper(base);
		let mut elements_tokens = proc_macro2::TokenStream::new();
		for other_base in &basis.0 {
			let other_element = element_name(other_base);
			elements_tokens.extend(if other_base == base {
				quote! {
					#other_element: 1.0,
				}
			} else {
				quote! {
					#other_element: 0.0,
				}
			});
		}
		constants_tokens.extend(quote! {
			pub const #element: Self = Self {
				#elements_tokens
			};
		});
	}
	constants_tokens
}

fn impl_normalized(basis: &Basis) -> TokenStream {
	let els = basis_names(basis);
	quote! {
		pub fn norm(self) -> f32 {
			(
				#(
					self.#els*self.#els
				)+*
			).sqrt()
		}

		pub fn normalize(self) -> Self {
			let norm = self.norm();
			self * norm.recip()
		}

		pub fn normalize_or_zero(self) -> Self {
			let norm = self.norm();
			if norm == 0.0 {
				Self::ZERO
			} else {
				self * norm.recip()
			}
		}
	}
}
