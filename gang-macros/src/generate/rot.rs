use quote::quote;

use crate::{
	util::{element_name, impl_mul_bases, Basis, LinearCombinations, Sign},
	MvKind,
};

pub(crate) fn impl_rotate(
	gen: &mut Vec<proc_macro2::TokenStream>,
	rotor_basis: &Basis,
	kind: MvKind,
	rhs_basis: &Basis,
) {
	let mut result = LinearCombinations::one();

	result = impl_mul_bases(
		&result,
		&rotor_basis
			.0
			.iter()
			.map(|r| (Sign::Pos, r.clone()))
			.collect::<Vec<_>>(),
	);

	result = impl_mul_bases(
		&result,
		&rhs_basis
			.0
			.iter()
			.map(|r| (Sign::Pos, r.clone()))
			.collect::<Vec<_>>(),
	);

	result = impl_mul_bases(
		&result,
		&rotor_basis
			.0
			.iter()
			.map(|r| {
				(
					match r.grade() / 2 % 2 {
						0 => Sign::Pos,
						1 => Sign::Neg,
						_ => unreachable!("maths broke"),
					},
					r.clone(),
				)
			})
			.collect::<Vec<_>>(),
	);

	let output_basis = rhs_basis;

	let mut rows = vec![];
	for term in &output_basis.0 {
		let term_name = element_name(&term);
		rows.push(quote! { #term_name : });
		if let Some(sum) = result.0.get(&term) {
			for (i, (sign, factors)) in sum.0.iter().enumerate() {
				match sign {
					Sign::Neg => rows.push(quote! {-}),
					Sign::Pos if i != 0 => rows.push(quote! {+}),
					_ => {}
				}
				let rot_factor = element_name(&factors[0]);
				let rhs_name = element_name(&factors[1]);
				let rotd_factor = element_name(&factors[2]);
				rows.push(quote! { self.#rot_factor*rhs.#rhs_name*self.#rotd_factor});
			}
		} else {
			rows.push(quote! { 0.0 });
		}
		rows.push(quote! {,});
	}

	gen.push(quote! {
		impl gang::Rotate<#kind> for Rot {
			type Output = #kind;
			fn rotate(self, rhs: #kind) -> Self::Output {
				Self::Output {
					#(#rows)*
				}
			}
		}
	});
}
