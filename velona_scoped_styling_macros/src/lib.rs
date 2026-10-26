mod propstack;

use proc_macro::TokenStream;
use proc_macro_error3::proc_macro_error;

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
#[proc_macro_error]
#[proc_macro]
pub fn scoped_propstack(input: TokenStream) -> TokenStream {
    propstack::propstack_impl(proc_macro2::TokenStream::from(input)).into()
}
