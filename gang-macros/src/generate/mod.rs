use quote::quote;

use crate::util::{element_name, element_name_upper};

mod rot;
pub(crate) use rot::impl_rotate;

pub fn constants(basis: &[Vec<u32>]) -> proc_macro2::TokenStream {
	let mut constants_tokens = proc_macro2::TokenStream::new();
	for base in basis {
		let element = element_name_upper(base);
		let mut elements_tokens = proc_macro2::TokenStream::new();
		for other_base in basis {
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