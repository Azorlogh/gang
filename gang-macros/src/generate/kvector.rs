use proc_macro2::TokenStream;
use quote::quote;

use super::impl_normalized;
use crate::{
	util::{basis_names, element_name, infer, mul_bases, Basis, Sign},
	MvKind,
};

pub fn kvector_methods(
	gen: &mut Vec<proc_macro2::TokenStream>,
	canonical_basis: &Basis,
	kvectors: &[Basis],
) {
	for (k, elements) in kvectors.iter().enumerate() {
		let kind = MvKind::KVector(k);
		let element_names = basis_names(elements);
		gen.push(quote! {
			impl std::ops::Mul<#kind> for #kind {
				type Output = #kind;
				fn mul(self, rhs: #kind) -> Self::Output {
					Self::Output {
						#(
							#element_names: self.#element_names * rhs.#element_names,
						)*
					}
				}
			}

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
		impl_neg(gen, kind, &elements);
	}

	let dim = canonical_basis.0.last().unwrap().0.len();
	for i in 0..kvectors.len() {
		for j in 0..kvectors.len() {
			if i + j <= dim {
				impl_wedge(gen, canonical_basis, i, &kvectors[i], j, &kvectors[j]);
			}
		}
	}
}

fn impl_wedge(
	gen: &mut Vec<proc_macro2::TokenStream>,
	canonical_basis: &Basis,
	lhs_grade: usize,
	lhs_basis: &Basis,
	rhs_grade: usize,
	rhs_basis: &Basis,
) {
	let mut combinations = mul_bases(&[&lhs_basis, &rhs_basis]);

	combinations
		.0
		.retain(|k, _| k.grade() == lhs_grade + rhs_grade);

	let output_kind = infer(combinations.0.keys().cloned().collect::<Vec<_>>());
	let output_basis = output_kind.get_elements(&canonical_basis.0);

	let mut rows = vec![];
	for term in output_basis {
		let term_name = element_name(&term);
		rows.push(quote! { #term_name : });
		if let Some(sum) = combinations.0.get(&term) {
			for (i, (sign, terms)) in sum.0.iter().enumerate() {
				match sign {
					Sign::Neg => rows.push(quote! {-}),
					Sign::Pos if i != 0 => rows.push(quote! {+}),
					_ => {}
				}
				let lhs_name = element_name(&terms[0]);
				let rhs_name = element_name(&terms[1]);
				rows.push(quote! { self.#lhs_name * rhs.#rhs_name});
			}
		} else {
			rows.push(quote! { 0.0 });
		}
		rows.push(quote! {,});
	}

	let lhs_kind = MvKind::KVector(lhs_grade);
	let rhs_kind = MvKind::KVector(rhs_grade);

	gen.push(quote! {
		impl gang::Wedge<#rhs_kind> for #lhs_kind {
			type Output = #output_kind;
			fn wedge(self, rhs: #rhs_kind) -> Self::Output {
				Self::Output {
					#(#rows)*
				}
			}
		}
	});
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

		impl std::ops::Add<f32> for #kind {
			type Output = #kind;
			fn add(self, rhs: f32) -> Self::Output {
				Self::Output {
					#(
						#element_names: self.#element_names + rhs,
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

		impl std::ops::AddAssign<f32> for #kind {
			fn add_assign(&mut self, rhs: f32) {
				#(
					self.#element_names += rhs;
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

		impl std::ops::Sub<f32> for #kind {
			type Output = #kind;
			fn sub(self, rhs: f32) -> Self::Output {
				Self::Output {
					#(
						#element_names: self.#element_names - rhs,
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

		impl std::ops::SubAssign<f32> for #kind {
			fn sub_assign(&mut self, rhs: f32) {
				#(
					self.#element_names -= rhs;
				)*
			}
		}
	});
}

fn impl_neg(gen: &mut Vec<proc_macro2::TokenStream>, kind: MvKind, basis: &Basis) {
	let element_names = basis_names(basis);
	gen.push(quote! {
		impl std::ops::Neg for #kind {
			type Output = #kind;
			fn neg(self) -> Self::Output {
				Self::Output {
					#(
						#element_names: -self.#element_names,
					)*
				}
			}
		}
	});
}

fn impl_methods(gen: &mut Vec<proc_macro2::TokenStream>, kind: MvKind, basis: &Basis) {
	let methods = vec![
		impl_sign(basis),
		impl_cmp(basis),
		impl_abs_diff_eq(basis),
		impl_rounding(basis),
		impl_normalized(basis),
		impl_to_array(basis),
	];
	gen.push(quote! {
		impl #kind {
			#(
				#methods
			)*
		}
	});
}

fn impl_sign(basis: &Basis) -> TokenStream {
	let els = basis_names(basis);
	quote! {
		pub fn abs(self) -> Self {
			Self {
				#(
					#els: self.#els.abs(),
				)*
			}
		}
	}
}

fn impl_cmp(basis: &Basis) -> TokenStream {
	let els = basis_names(basis);
	quote! {
		pub fn min(self, rhs: Self) -> Self {
			Self {
				#(
					#els: self.#els.min(rhs.#els),
				)*
			}
		}

		pub fn max(self, rhs: Self) -> Self {
			Self {
				#(
					#els: self.#els.max(rhs.#els),
				)*
			}
		}
	}
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

fn impl_rounding(basis: &Basis) -> TokenStream {
	let els = basis_names(basis);
	quote! {
		pub fn floor(self) -> Self {
			Self {
				#(
					#els: self.#els.floor(),
				)*
			}
		}
		pub fn round(self) -> Self {
			Self {
				#(
					#els: self.#els.round(),
				)*
			}
		}
		pub fn ceil(self) -> Self {
			Self {
				#(
					#els: self.#els.ceil(),
				)*
			}
		}
	}
}

fn impl_to_array(basis: &Basis) -> TokenStream {
	let els = basis_names(basis);
	let basis_len = basis.0.len();
	let indices = (0..basis_len).collect::<Vec<_>>();
	quote! {
		pub fn from_array(arr: [f32; #basis_len]) -> Self {
			Self::new(
				#(
					arr[#indices],
				)*
			)
		}

		pub fn to_array(self) -> [f32; #basis_len] {
			[
				#(
					self.#els,
				)*
			]
		}
	}
}
