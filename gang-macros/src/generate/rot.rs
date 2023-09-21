use quote::quote;

use crate::{
	util::{element_name, element_name_upper, mul_bases},
	MvKind,
};

pub(crate) fn impl_rotate(
	gen: &mut Vec<proc_macro2::TokenStream>,
	rotor_basis: &[Vec<u32>],
	kind: MvKind,
	rhs_basis: &[Vec<u32>],
) -> proc_macro2::TokenStream {
	let mut tokens = proc_macro2::TokenStream::new();

	// let (calc_map, output_kind) = mul_bases(lhs.1, rhs.1);

	// tokens.extend(quote! {
	// 	impl Rotate<#kind> for Rot {
	// 		type Output = #kind;

	// 		fn rotate(self, rhs: #kind) -> Self::Output {
	// 			#kind {

	// 			}
	// 		}
	// 	}
	// });

	tokens
}
