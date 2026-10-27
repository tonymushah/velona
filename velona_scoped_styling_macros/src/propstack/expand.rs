use std::collections::HashMap;

use proc_macro_error3::abort;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Ident};

use crate::propstack::parsing::{
    Property, PropertyStateAtt, RawClasses, SelectorDeclaration, SelectorDeclarations,
};

pub fn expand(input: SelectorDeclarations) -> TokenStream {
    let declarations = input.declarations.into_iter().map(expand_declaration);

    let declaration = property_stack_declaration();

    quote! {{
        #declaration

        #(#declarations)*

        property_stack
    }}
}

fn property_stack_declaration() -> TokenStream {
    quote! {
        let mut property_stack = <ScopedPropstack as core::default::Default>::default();
    }
}

fn create_classes_declaration(classes: RawClasses) -> TokenStream {
    let classes = classes.classes.iter();
    quote! {
        const CLASSES: &[&str] = &[#(#classes),*];
    }
}

fn make_selector(state: Option<&PropertyStateAtt>) -> TokenStream {
    let selector = quote! {
        let mut selector = Selector::classes(CLASSES);
    };
    let val = state.iter().flat_map(|state| {
        state.values.iter().map(|value| match value {
            crate::propstack::parsing::PropertyStateAttValue::Single(ident) => {
                if ident == "hover" {
                    quote! {
                        selector = selector.with_hovered(true);
                    }
                } else if ident == "active" {
                    quote! {
                        selector = selector.with_active(true);
                    }
                } else if ident == "disabled" {
                    quote! {
                        selector = selector.with_disabled(true);
                    }
                } else if ident == "focused" {
                    quote! {
                        selector = selector.with_focused(true);
                    }
                } else {
                    abort!(
                        ident,
                        "Invalid ident, excepted `hover` or `active` or `disabled` or `focused`"
                    )
                }
            }
            crate::propstack::parsing::PropertyStateAttValue::Parenthed {
                parent,
                paren: _,
                inner_value: ident,
            } => {
                if parent == "not" {
                    if ident == "hover" {
                        quote! {
                            selector = selector.with_hovered(false);
                        }
                    } else if ident == "active" {
                        quote! {
                            selector = selector.with_active(false);
                        }
                    } else if ident == "disabled" {
                        quote! {
                            selector = selector.with_disabled(false);
                        }
                    } else if ident == "focused" {
                        quote! {
                            selector = selector.with_focused(false);
                        }
                    } else {
                        abort!(
                            ident,
                            "Invalid ident, excepted `hover` or `active` or `disabled` or `focused`"
                        )
                    }
                } else {
                    abort!(parent, "Invalid ident, expected `not`")
                }
            }
        })
    });
    quote! {{
        #selector

        #(#val)*

        selector
    }}
}

// TODO Dedup state declaration
fn expand_property_declaration(
    property: &Property,
    state_map: &HashMap<&Option<PropertyStateAtt>, Ident>,
) -> TokenStream {
    let Some(state) = state_map.get(&property.state) else {
        abort!(property.property, "Cannont find internal state ident");
    };
    match &property.property {
        Expr::Closure(value) => {
            let mut opt = false;
            let mut clones = Vec::<TokenStream>::new();
            for att in &property.attrs {
                if att.name == "opt" {
                    if opt {
                        abort!(att.name, "Cannot have `@opt` attribute twice");
                    } else {
                        opt = true;
                    }
                } else if att.name == "clone" {
                    for to_clone in att.parenthesises_content.iter().flatten() {
                        clones.push(quote! {
                            let #to_clone = core::clone::Clone::clone(&#to_clone);
                        });
                    }
                } else {
                    abort!(att.name, "Invalid name, expeced `opt` or `clone`");
                }
            }
            if value.inputs.is_empty() {
                let prop_fn = if opt {
                    quote! {
                        property_stack = property_stack.prop_opt(#state.clone(), move|_| prop_fn());
                    }
                } else {
                    quote! {
                        property_stack = property_stack.prop(#state.clone(), move|_| prop_fn());
                    }
                };
                quote! {{
                    #(#clones)*

                    let prop_fn = #value;

                    #prop_fn
                }}
            } else {
                if opt {
                    quote! {
                        property_stack = property_stack.prop_opt(#state.clone(), #value)
                    }
                } else {
                    quote! {
                        property_stack = property_stack.prop(#state.clone(), #value)
                    }
                }
            }
        }
        value => {
            if !property.attrs.is_empty() {
                abort!(
                    property.attrs[0].suffix,
                    "A non-closure property cannot have a `@` attribute"
                );
            } else {
                quote! {
                    property_stack = property_stack.static_prop(#state.clone(), #value);
                }
            }
        }
    }
}

fn expand_declaration(input: SelectorDeclaration) -> TokenStream {
    let classes = create_classes_declaration(input.classes);

    let state_map = input
        .properties
        .properties
        .iter()
        .enumerate()
        .map(|a| (&a.1.state, format_ident!("state_{}", a.0)))
        .collect::<HashMap<_, _>>();

    let states = state_map.iter().map(|(state, name)| {
        let selector = make_selector(state.as_ref());
        quote! {
            let #name = #selector;
        }
    });

    let properties = input
        .properties
        .properties
        .iter()
        .map(|a| expand_property_declaration(a, &state_map));

    quote! {{
        #classes

        #(#states)*

        #(#properties)*

    }}
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_property_stack_declaration() {
        assert_eq!(
            property_stack_declaration().to_string(),
            "let mut property_stack = < ScopedPropstack as core :: default :: Default > :: default () ;"
        );
    }

    #[test]
    fn test_classes_declaration() {
        let raw_classes: RawClasses = parse_quote!(["a", "b", "c"]);

        let val = create_classes_declaration(raw_classes).to_string();
        assert_eq!(val, "const CLASSES : & [& str] = & [\"a\" , \"b\" , \"c\"] ;");
    }
}
