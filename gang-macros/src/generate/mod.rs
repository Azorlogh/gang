use quote::quote;

use crate::util::{element_name, element_name_upper, Basis};

mod kvector;
mod rot;
pub(crate) use kvector::kvector_methods;
pub(crate) use rot::impl_rotate;

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
