use quote::{quote, ToTokens};

use crate::util::{
	element_name, impl_mul_bases, Basis, LinearCombination, LinearCombinations, Sign,
};

pub(crate) fn impl_to_matrix(
	gen: &mut Vec<proc_macro2::TokenStream>,
	rotor_basis: &Basis,
	rhs_basis: &Basis,
) {
	let kind = crate::util::infer(rhs_basis.0.clone());

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

	let nb_els = rhs_basis.0.len();

	let mut matrix: Vec<LinearCombination> = vec![LinearCombination::zero(); nb_els * nb_els];

	for (i, element) in rhs_basis.0.iter().enumerate() {
		let comb = &result.0[element];
		for term in &comb.0 {
			let j = rhs_basis.0.iter().position(|el| el == &term.1[1]).unwrap();
			// println!("filling: {:?} {:?}", i, j);
			matrix[i + j * nb_els] // column-major
				.0
				.push((term.0, vec![term.1[0].clone(), term.1[2].clone()]));
		}
	}

	// println!("\n\nhello again!!!");

	let mut els = vec![];
	for sum in matrix {
		for (i, (sign, factors)) in sum.0.iter().enumerate() {
			match sign {
				Sign::Neg => els.push(quote! {-}),
				Sign::Pos if i != 0 => els.push(quote! {+}),
				_ => {}
			}
			let rot_factor = element_name(&factors[0]);
			let rotd_factor = element_name(&factors[1]);
			els.push(quote! { self.#rot_factor*self.#rotd_factor });
		}
		els.push(quote! { , });
	}

	let method_name = proc_macro2::Ident::new(
		&format!(
			"{}_rotation_matrix",
			kind.into_token_stream().to_string().to_lowercase()
		),
		proc_macro2::Span::call_site(),
	);
	gen.push(quote! {
		impl Rot {
			pub fn #method_name(self) -> Vec<f32> {
				vec![
					#(#els)*
				]
			}
		}
	});
}

pub(crate) fn impl_rotate(
	gen: &mut Vec<proc_macro2::TokenStream>,
	rotor_basis: &Basis,
	rhs_basis: &Basis,
) {
	let kind = crate::util::infer(rhs_basis.0.clone());
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
