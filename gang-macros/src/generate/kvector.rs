use proc_macro2::TokenStream;
use quote::quote;

use crate::{
	util::{basis_names, Basis},
	MvKind,
};

pub fn kvector_methods(gen: &mut Vec<proc_macro2::TokenStream>, kvectors: &[Basis]) {
	for (k, elements) in kvectors.iter().enumerate() {
		let kind = MvKind::KVector(k);
		let element_names = basis_names(elements);
		gen.push(quote! {
			impl std::ops::Mul<f32> for #kind {
				type Output = #kind;
				fn mul(self, rhs: f32) -> Self::Output {
					Self::Output {
						#(
							#element_names: self.#element_names * rhs,
						)*
					}
				}
			}
		});

		impl_methods(gen, kind, &elements);

		impl_add(gen, kind, &elements);
		impl_sub(gen, kind, &elements);
	}
}

fn impl_add(gen: &mut Vec<proc_macro2::TokenStream>, kind: MvKind, basis: &Basis) {
	let element_names = basis_names(basis);
	gen.push(quote! {
		impl std::ops::Add<#kind> for #kind {
			type Output = #kind;
			fn add(self, rhs: #kind) -> Self::Output {
				Self::Output {
					#(
						#element_names: self.#element_names + rhs.#element_names,
					)*
				}
			}
		}

		impl std::ops::AddAssign<#kind> for #kind {
			fn add_assign(&mut self, rhs: #kind) {
				#(
					self.#element_names += rhs.#element_names;
				)*
			}
		}
	});
}

fn impl_sub(gen: &mut Vec<proc_macro2::TokenStream>, kind: MvKind, basis: &Basis) {
	let element_names = basis_names(basis);
	gen.push(quote! {
		impl std::ops::Sub<#kind> for #kind {
			type Output = #kind;
			fn sub(self, rhs: #kind) -> Self::Output {
				Self::Output {
					#(
						#element_names: self.#element_names - rhs.#element_names,
					)*
				}
			}
		}

		impl std::ops::SubAssign<#kind> for #kind {
			fn sub_assign(&mut self, rhs: #kind) {
				#(
					self.#element_names -= rhs.#element_names;
				)*
			}
		}
	});
}

fn impl_methods(gen: &mut Vec<proc_macro2::TokenStream>, kind: MvKind, basis: &Basis) {
	let methods = vec![impl_abs_diff_eq(basis)];
	gen.push(quote! {
		impl #kind {
			#(
				#methods
			)*
		}
	});
}

fn impl_abs_diff_eq(basis: &Basis) -> TokenStream {
	let els = basis_names(basis);
	quote! {
		pub fn abs_diff_eq(self, rhs: Self, max_abs_diff: f32) -> bool {
			#(
				(self.#els - rhs.#els).abs() < max_abs_diff
			)&&*
		}
	}
}
