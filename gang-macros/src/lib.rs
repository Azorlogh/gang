use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::Ident;
use util::{element_name, infer, Basis, Element, Sign};

#[proc_macro]
pub fn gang(input: TokenStream) -> TokenStream {
	let ast: syn::LitInt = syn::parse(input).unwrap();

	let dim = ast.base10_parse::<u32>().unwrap();

	// basis elements - products of basis vectors in increasing order
	let canonical_basis = {
		let mut basis: Vec<Element> = Vec::new();
		for i in 0..2_u32.pow(dim) {
			let mut elem = Vec::new();
			for j in 0..2_u32.pow(dim) {
				if (i >> j) % 2 == 1 {
					elem.push(j)
				}
			}
			basis.push(Element(elem));
		}
		basis.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0)));
		Basis(basis)
	};

	// k-vectors - grade k multivectors
	let mut kvectors: Vec<Basis> = Vec::new();
	for g in 0..=dim {
		kvectors.push(Basis(
			canonical_basis
				.0
				.iter()
				.cloned()
				.filter(|c| c.grade() as u32 == g)
				.collect::<Vec<_>>(),
		))
	}

	// rotor - sum of 2k-vectors
	let mut rotor_basis = Basis(Vec::new());
	for c in &canonical_basis.0 {
		if c.grade() % 2 == 0 {
			rotor_basis.0.push(c.clone());
		}
	}

	// code output
	let mut gen: Vec<proc_macro2::TokenStream> = vec![];

	let gen_specialized_mv_struct =
		|gen: &mut Vec<proc_macro2::TokenStream>, name: &Ident, basis: &Basis, constants: bool| {
			let bases = basis
				.0
				.iter()
				.map(|c| format_ident!("{}", element_name(c)))
				.collect::<Vec<_>>();

			let constants_tokens = if constants {
				generate::constants(basis)
			} else {
				proc_macro2::TokenStream::new()
			};

			let maybe_bevy_reflect = if cfg!(feature = "bevy_reflect") {
				quote! {
					#[derive(bevy_reflect::Reflect)]
				}
			} else {
				proc_macro2::TokenStream::new()
			};

			gen.push(quote! {
				#[derive(Clone, Copy, PartialEq, Debug)]
				#maybe_bevy_reflect
				pub struct #name {
					#(
						pub #bases: f32,
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
		gen_specialized_mv_struct(&mut gen, &name, &basis, true);
		let bases = basis
			.0
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
		let other_element_names = rotor_basis
			.0
			.iter()
			.filter(|c| !c.0.is_empty()) // e is written explicitly below
			.map(|c| format_ident!("{}", element_name(&c)))
			.collect::<Vec<_>>();
		gen_specialized_mv_struct(&mut gen, &format_ident!("Rot"), &rotor_basis, false);
		gen.push(quote! {
			impl Rot {
				pub const IDENTITY: Self = Self {
					e: 1.0,
					#(
						#other_element_names: 0.0,
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
			.0
			.iter()
			.filter_map(|e| (e.grade() == 2).then_some(element_name(e)))
			.collect();
		let missing_elements: Vec<_> = rotor_basis
			.0
			.iter()
			.filter_map(|e| (e.grade() != 0 && e.grade() != 2).then_some(element_name(e)))
			.collect();
		gen.push(quote! {
			impl Rot {
				pub fn from_v2_angle(v2: V2, angle: f32) -> Self {
					let a = angle / 2.0;
					let (s, c) = a.sin_cos();
					Self {
						e: c,
						#(
							#v2_elements: -v2.#v2_elements*s,
						)*
						#(
							#missing_elements: 0.0,
						)*
					}
				}
			}
		});

		if kvectors[2].0.len() == 1 {
			let single_element = element_name(&kvectors[2].0[0]);
			// angle() can probably work in 3d too, but I don't think it can work in 4D
			gen.push(quote! {
				impl Rot {
					pub fn from_angle(angle: f32) -> Self {
						let a = angle / 2.0;
						let (s, c) = a.sin_cos();
						Self {
							e: c,
							#single_element: -s
						}
					}

					pub fn angle(self) -> f32 {
						2.0*(-self.#single_element).atan2(self.e)
					}
				}
			});
		}
	}

	generate::rotor_methods(&mut gen, &rotor_basis);

	impl_mul(
		&mut gen,
		&canonical_basis.0,
		(format_ident!("Rot"), &rotor_basis),
		(format_ident!("Rot"), &rotor_basis),
	);

	generate::kvector_methods(&mut gen, &canonical_basis, &kvectors);

	for elements in &kvectors {
		generate::impl_rotate(&mut gen, &rotor_basis, elements);
	}

	generate::impl_to_matrix(&mut gen, &rotor_basis, &kvectors[1]);

	quote! {
		#(#gen)*
	}
	.into()
}

mod generate;
mod util;

fn impl_mul(
	gen: &mut Vec<proc_macro2::TokenStream>,
	elements: &[Element],
	lhs: (Ident, &Basis),
	rhs: (Ident, &Basis),
) {
	let lhs_name = lhs.0;
	let rhs_name = rhs.0;

	let (calc_map, output_kind) = {
		let map = util::mul_bases(&[&lhs.1, &rhs.1]);
		(
			map.0.clone(),
			infer(map.0.keys().cloned().collect::<Vec<_>>()),
		)
	};

	let output_basis = output_kind.get_elements(elements);

	let mut rows = vec![];
	for term in output_basis {
		let term_name = element_name(&term);
		rows.push(quote! { #term_name : });
		if let Some(sum) = calc_map.get(&term) {
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

	gen.push(quote! {
		impl std::ops::Mul<#rhs_name> for #lhs_name {
			type Output = #output_kind;
			fn mul(self, rhs: #rhs_name) -> Self::Output {
				Self::Output {
					#(#rows)*
				}
			}
		}
	});
}

// type Element = Vec<u32>;

#[derive(Debug, Clone, Copy)]
enum MvKind {
	KVector(usize),
	Rotor,
	General,
}

impl MvKind {
	pub fn get_elements(&self, elements: &[Element]) -> Vec<Element> {
		match self {
			MvKind::KVector(k) => elements
				.iter()
				.cloned()
				.filter(|e| e.0.len() == *k)
				.collect(),
			MvKind::Rotor => elements
				.iter()
				.cloned()
				.filter(|e| e.0.len() % 2 == 0)
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
