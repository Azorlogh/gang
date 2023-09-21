//!
//! Terminology:
//! 	basis: subset of possible products of basis vectors in the GA
//!

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::Ident;
use util::{basis_names, element_name_upper};

use crate::util::{element_name, infer, mul_bases};

#[proc_macro]
pub fn gang(input: TokenStream) -> TokenStream {
	let ast: syn::LitInt = syn::parse(input).unwrap();

	let dim = ast.base10_parse::<u32>().unwrap();

	// basis elements - products of basis vectors in increasing order
	let mut elements = Vec::new();
	for i in 0..2_u32.pow(dim) {
		let mut elem = Vec::new();
		for j in 0..2_u32.pow(dim) {
			if (i >> j) % 2 == 1 {
				elem.push(j)
			}
		}
		elements.push(elem);
	}
	elements.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(&b)));

	// k-vectors - grade k multivectors
	let mut kvectors = Vec::new();
	for g in 0..=dim {
		kvectors.push(
			elements
				.iter()
				.cloned()
				.filter(|c| c.len() as u32 == g)
				.collect::<Vec<_>>(),
		)
	}

	// rotor - sum of 2k-vectors
	let mut rotor_basis = Vec::new();
	for c in &elements {
		if c.len() % 2 == 0 {
			rotor_basis.push(c.clone());
		}
	}

	// code output
	let mut gen: Vec<proc_macro2::TokenStream> = vec![];

	let gen_specialized_mv_struct =
		|gen: &mut Vec<proc_macro2::TokenStream>, name: &Ident, basis: &[Vec<u32>]| {
			let bases = basis
				.iter()
				.map(|c| format_ident!("{}", element_name(&c)))
				.collect::<Vec<_>>();

			let constants_tokens = generate::constants(basis);

			gen.push(quote! {
				#[derive(Clone, Copy, PartialEq, bevy_reflect::Reflect, Debug)]
				struct #name {
					#(
						#bases: f32,
					)*
				}

				impl #name {
					#constants_tokens

					pub fn new(#(#bases: f32,)*) -> Self {
						Self {
							#(
								#bases,
							)*
						}
					}
				}
			});
		};

	// generate k-vectors
	for (k, basis) in kvectors.iter().enumerate() {
		let name = format_ident!("V{k}");
		gen_specialized_mv_struct(&mut gen, &name, &basis);
		let bases = basis
			.iter()
			.map(|c| format_ident!("{}", element_name(&c)))
			.collect::<Vec<_>>();
		gen.push(quote! {
			impl #name {
				pub const ZERO: Self = Self {
					#(
						#bases: 0.0,
					)*
				};

				pub const ONE: Self = Self {
					#(
						#bases: 1.0,
					)*
				};
			}

			impl Default for #name {
				fn default() -> Self {
					Self::ZERO
				}
			}
		});
	}

	{
		let element_names = rotor_basis
			.iter()
			.filter(|c| !c.is_empty())
			.map(|c| format_ident!("{}", element_name(&c)))
			.collect::<Vec<_>>();
		gen_specialized_mv_struct(&mut gen, &format_ident!("Rot"), &rotor_basis);
		gen.push(quote! {
			impl Rot {
				pub const IDENTITY: Self = Self {
					e: 1.0,
					#(
						#element_names: 0.0,
					)*
				};
			}

			impl Default for Rot {
				fn default() -> Self {
					Self::IDENTITY
				}
			}
		});

		let v2_elements: Vec<_> = rotor_basis
			.iter()
			.filter_map(|e| (e.len() == 2).then_some(element_name(e)))
			.collect();
		let missing_elements: Vec<_> = rotor_basis
			.iter()
			.filter_map(|e| (e.len() != 0 && e.len() != 2).then_some(element_name(e)))
			.collect();
		gen.push(quote! {
			impl Rot {
				fn from_v2_angle(v2: V2, angle: f32) -> Self {
					let a = angle / 2.0;
					let (s, c) = a.sin_cos();
					Self {
						e: c,
						#(
							#v2_elements: v2.#v2_elements*s,
						)*
						#(
							#missing_elements: 0.0,
						)*
					}
				}
			}
		});
	}

	impl_mul(
		&mut gen,
		&elements,
		(format_ident!("Rot"), &rotor_basis),
		(format_ident!("Rot"), &rotor_basis),
	);

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

		impl_add(&mut gen, kind, &elements);
		impl_sub(&mut gen, kind, &elements);

		generate::impl_rotate(&mut gen, &rotor_basis, kind, &elements);
	}

	quote! {
		#(#gen)*
	}
	.into()
}

mod generate;
mod util;

type Basis = [Vec<u32>];

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

fn impl_mul(
	gen: &mut Vec<proc_macro2::TokenStream>,
	elements: &[Element],
	lhs: (Ident, &Basis),
	rhs: (Ident, &Basis),
) {
	let lhs_name = lhs.0;
	let rhs_name = rhs.0;

	let (calc_map, output_kind) = mul_bases(lhs.1, rhs.1);

	let output_basis = output_kind.get_elements(elements);

	let mut rows = vec![];
	for term in output_basis {
		let term_name = element_name(&term);
		rows.push(quote! { #term_name : });
		if let Some(sum) = calc_map.get(&term) {
			for (i, (sign, lhs, rhs)) in sum.iter().enumerate() {
				if *sign {
					rows.push(quote! {-})
				} else if i != 0 {
					rows.push(quote! {+})
				}
				let lhs_name = element_name(&lhs);
				let rhs_name = element_name(&rhs);
				rows.push(quote! { self.#lhs_name * rhs.#rhs_name})
			}
		} else {
			rows.push(quote! { 0.0 })
		}
		rows.push(quote! {,})
	}

	gen.push(quote! {
		impl std::ops::Mul<#lhs_name> for #rhs_name {
			type Output = #output_kind;
			fn mul(self, rhs: #lhs_name) -> Self::Output {
				Self::Output {
					#(#rows)*
				}
			}
		}
	});
}

type Element = Vec<u32>;

#[derive(Clone, Copy)]
enum MvKind {
	KVector(usize),
	Rotor,
	General,
}

impl MvKind {
	pub fn get_elements(&self, elements: &[Element]) -> Vec<Element> {
		match self {
			MvKind::KVector(k) => elements.iter().cloned().filter(|e| e.len() == *k).collect(),
			MvKind::Rotor => elements
				.iter()
				.cloned()
				.filter(|e| e.len() % 2 == 0)
				.collect(),
			MvKind::General => elements.to_owned(),
		}
	}
}

impl ToTokens for MvKind {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		tokens.append(match self {
			MvKind::KVector(k) => format_ident!("V{k}"),
			MvKind::Rotor => format_ident!("Rot"),
			MvKind::General => format_ident!("Mv"),
		});
	}
}
