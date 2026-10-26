mod propstack;

use proc_macro::TokenStream;

// ```
// propstack! {
//	["aasdsad", "sadaddasd"] {
//		Background::Color(___);
//		move |_| {};
//		#[active, not(disabled)]
//		@opt
//		@clone(val1, val2, val3, val4, val5)
//		move |_| {};
//	}
// }
// ```
#[proc_macro]
pub fn propstack(input: TokenStream) -> TokenStream {
    propstack::propstack_impl(proc_macro2::TokenStream::from(input)).into()
}
