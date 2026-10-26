use proc_macro2::TokenStream;

use crate::propstack::parsing::SelectorDeclarations;

mod expand;
mod parsing;

pub fn propstack_impl(input: TokenStream) -> TokenStream {
    let parsed: SelectorDeclarations = match syn::parse2(input) {
        Ok(v) => v,
        Err(err) => return err.into_compile_error(),
    };
    expand::expand(parsed)
}
