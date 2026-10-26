// use proc_macro_error3::{abort, emit_error, emit_warning};
use syn::{
    Expr, Ident, LitStr, Token, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

pub struct RawClass {
    pub value: LitStr,
    pub delimited: Option<Token![,]>,
}

impl Parse for RawClass {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            value: input.parse()?,
            delimited: input.parse()?,
        })
    }
}

pub struct RawClasses {
    pub braquet: token::Bracket,
    pub classes: Punctuated<LitStr, Token![,]>,
}

impl RawClasses {
    pub fn get_classes_string(&self) -> Vec<String> {
        self.classes.iter().map(|class| class.value()).collect()
    }
}

impl Parse for RawClasses {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let braquet = bracketed!(content in input);

        Ok(Self {
            braquet,
            classes: Punctuated::parse_terminated(&content)?,
        })
    }
}

pub struct PropertyAAttribute {
    pub suffix: Token![@],
    pub name: Ident,
    pub parenthesises: Option<token::Paren>,
    pub parenthesises_content: Option<Punctuated<Ident, Token![,]>>,
}

impl Parse for PropertyAAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let suffix = input.parse()?;
        let name = input.parse()?;
        let (paren, content) = if input.peek(token::Paren) {
            let content;
            let paren = parenthesized!(content in input);
            if content.is_empty() {
                (Some(paren), None)
            } else {
                (Some(paren), Some(Punctuated::parse_terminated(&content)?))
            }
        } else {
            (None, None)
        };
        Ok(Self {
            suffix,
            name,
            parenthesises: paren,
            parenthesises_content: content,
        })
    }
}

pub enum PropertyStateAttValue {
    Single(Ident),
    Parenthed {
        parent: Ident,
        paren: token::Paren,
        inner_value: Ident,
    },
}

impl Parse for PropertyStateAttValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let start_value = input.parse()?;
        if input.peek(token::Paren) {
            let content;
            let paren = parenthesized!(content in input);
            Ok(Self::Parenthed {
                parent: start_value,
                paren,
                inner_value: content.parse()?,
            })
        } else {
            Ok(Self::Single(start_value))
        }
    }
}

pub struct PropertyStateAtt {
    pub pound_token: Token![#],
    pub bracket: token::Bracket,
    pub values: Punctuated<PropertyStateAttValue, Token![,]>,
}

impl Parse for PropertyStateAtt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            pound_token: input.parse()?,
            bracket: bracketed!(content in input),
            values: Punctuated::parse_terminated(&content)?,
        })
    }
}

pub struct Property {
    pub state: Option<PropertyStateAtt>,
    pub attrs: Vec<PropertyAAttribute>,
    pub property: Expr,
}

impl Parse for Property {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let state: Option<PropertyStateAtt> = if input.peek(Token![#]) {
            Some(input.parse()?)
        } else {
            None
        };
        let mut attrs = Vec::<PropertyAAttribute>::new();
        loop {
            if input.peek(Token![@]) {
                attrs.push(input.parse()?);
            } else {
                break;
            }
        }
        Ok(Self {
            state,
            attrs,
            property: input.parse()?,
        })
    }
}

pub struct Properties {
    pub braces: token::Brace,
    pub properties: Punctuated<Property, Token![;]>,
}

impl Parse for Properties {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let braces = braced!(content in input);

        Ok(Self {
            braces,
            properties: Punctuated::parse_terminated(&content)?,
        })
    }
}

pub struct SelectorDeclaration {
    pub classes: RawClasses,
    pub properties: Properties,
}

impl Parse for SelectorDeclaration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            classes: input.parse()?,
            properties: input.parse()?,
        })
    }
}

pub struct SelectorDeclarations {
    pub declarations: Vec<SelectorDeclaration>,
}

impl Parse for SelectorDeclarations {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut declarations = Vec::<SelectorDeclaration>::new();
        while !input.is_empty() {
            declarations.push(input.parse()?);
        }
        Ok(Self { declarations })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_class_parsing() {
        let classes: RawClasses = parse_quote!(["t1", "c2"]);
        assert_eq!(classes.get_classes_string(), ["t1", "c2"])
    }

    #[test]
    #[should_panic = "expected `,`"]
    fn test_class_bad_parsing() {
        let _: RawClasses = parse_quote!(["t1", "c2" "c3"]);
        // assert_eq!(classes.get_classes_string(), ["t1", "c2"])
    }

    #[test]
    fn test_a_opt_attributes() {
        let value: PropertyAAttribute = parse_quote!(@opt);

        assert_eq!(value.name, "opt");
    }

    #[test]
    fn test_a_value_attributes() {
        let value: PropertyAAttribute = parse_quote!(@value(var1, var2));

        assert_eq!(value.name, "value");
        assert!(value.parenthesises.is_some());
        assert_eq!(value.parenthesises_content.as_ref().unwrap()[0], "var1");
        assert_eq!(value.parenthesises_content.as_ref().unwrap()[1], "var2");
    }

    #[test]
    fn test_a_value_empty_paren_attributes() {
        let value: PropertyAAttribute = parse_quote!(@value());

        assert_eq!(value.name, "value");
        assert!(value.parenthesises.is_some());
        assert!(value.parenthesises_content.is_none());
    }

    #[test]
    fn test_property_parsing_single_closure() {
        let value: Property = parse_quote!(|| {});

        assert!(value.attrs.is_empty());
        assert!(value.state.is_none());
        assert!(matches!(value.property, Expr::Closure(_)));
    }

    #[test]
    fn test_property_parsing_single_value() {
        let value: Property = parse_quote!(Background::Color(WHITE));

        assert!(value.attrs.is_empty());
        assert!(value.state.is_none());

        if let Expr::Call(call) = &value.property
            && let Expr::Path(path) = &*call.func
        {
            assert_eq!(path.path.segments.get(0).unwrap().ident, "Background");
            assert_eq!(path.path.segments.get(1).unwrap().ident, "Color");
            if let Expr::Path(path) = call.args.get(0).unwrap() {
                assert!(
                    path.path.segments.get(0).unwrap().ident == "WHITE",
                    "The argument should be `WHITE`"
                );
            }
        } else {
            panic!("Unreachable statement")
        }
    }
    #[test]
    fn test_property_parsing_single_ident() {
        let value: Property = parse_quote!(some_var);

        assert!(value.attrs.is_empty());
        assert!(value.state.is_none());
        assert!(matches!(value.property, Expr::Path(_)));
    }

    #[test]
    fn test_property_parsing_closure_with_attributes() {
        let value: Property = parse_quote!(
            #[active, not(disabled)]
            @clone(some_val)
            || some_val()
        );

        {
            let state = value.state.as_ref().unwrap();
            if let PropertyStateAttValue::Single(name) = state.values.get(0).unwrap() {
                assert_eq!(name, "active")
            } else {
                unreachable!()
            }
            if let PropertyStateAttValue::Parenthed {
                parent,
                paren: _,
                inner_value,
            } = state.values.get(1).unwrap()
            {
                assert_eq!(parent, "not");
                assert_eq!(inner_value, "disabled");
            } else {
                unreachable!()
            }
        }
        assert_eq!(value.attrs.len(), 1);

        assert_eq!(value.attrs[0].name, "clone");
        assert_eq!(
            value.attrs[0].parenthesises_content.as_ref().unwrap()[0],
            "some_val"
        );

        assert!(value.state.is_some());

        assert!(matches!(value.property, Expr::Closure(_)));
    }
    #[test]
    fn test_properties_parsing() {
        let values: Properties = parse_quote!({
            some_value();
            #[active, not(disabled)]
            @clone(some_val)
            || some_val()
        });

        {
            let value = values.properties.get(0).unwrap();
            assert!(value.attrs.is_empty());
            assert!(value.state.is_none());
            assert!(matches!(value.property, Expr::Call(_)));
        }
        {
            let value = values.properties.get(1).unwrap();
            {
                let state = value.state.as_ref().unwrap();
                if let PropertyStateAttValue::Single(name) = state.values.get(0).unwrap() {
                    assert_eq!(name, "active")
                } else {
                    unreachable!()
                }
                if let PropertyStateAttValue::Parenthed {
                    parent,
                    paren: _,
                    inner_value,
                } = state.values.get(1).unwrap()
                {
                    assert_eq!(parent, "not");
                    assert_eq!(inner_value, "disabled");
                } else {
                    unreachable!()
                }
            }
            assert_eq!(value.attrs.len(), 1);

            assert_eq!(value.attrs[0].name, "clone");
            assert_eq!(
                value.attrs[0].parenthesises_content.as_ref().unwrap()[0],
                "some_val"
            );

            assert!(value.state.is_some());

            assert!(matches!(value.property, Expr::Closure(_)));
        }
    }

    #[test]
    fn test_selector_declaration_parsing() {
        let declaration: SelectorDeclaration = parse_quote!(
            ["class1", "class2"] {
                //
                some_value();
                //
                #[active, not(disabled)]
                @clone(some_val)
                || some_val()
            }
        );

        let classes = declaration.classes;

        assert_eq!(classes.classes.len(), 2);

        assert_eq!(classes.classes.get(0).unwrap().value(), "class1");

        assert_eq!(classes.classes.get(1).unwrap().value(), "class2");

        let values = declaration.properties;
        {
            let value = values.properties.get(0).unwrap();
            assert!(value.attrs.is_empty());
            assert!(value.state.is_none());
            assert!(matches!(value.property, Expr::Call(_)));
        }
        {
            let value = values.properties.get(1).unwrap();
            {
                let state = value.state.as_ref().unwrap();
                if let PropertyStateAttValue::Single(name) = state.values.get(0).unwrap() {
                    assert_eq!(name, "active")
                } else {
                    unreachable!()
                }
                if let PropertyStateAttValue::Parenthed {
                    parent,
                    paren: _,
                    inner_value,
                } = state.values.get(1).unwrap()
                {
                    assert_eq!(parent, "not");
                    assert_eq!(inner_value, "disabled");
                } else {
                    unreachable!()
                }
            }
            assert_eq!(value.attrs.len(), 1);

            assert_eq!(value.attrs[0].name, "clone");
            assert_eq!(
                value.attrs[0].parenthesises_content.as_ref().unwrap()[0],
                "some_val"
            );

            assert!(value.state.is_some());

            assert!(matches!(value.property, Expr::Closure(_)));
        }
    }

    #[test]
    fn test_selector_declarations_parsing() {
        let declarations: SelectorDeclarations = parse_quote! {
            ["class1", "class2"] {
                //
                some_value();
                //
                #[active, not(disabled)]
                @clone(some_val)
                || some_val()
            }
            ["class2"] {
                some_val
            }
        };

        assert_eq!(declarations.declarations.len(), 2);
        {
            let declaration = &declarations.declarations[0];

            let classes = &declaration.classes;

            assert_eq!(classes.classes.len(), 2);

            assert_eq!(classes.classes.get(0).unwrap().value(), "class1");

            assert_eq!(classes.classes.get(1).unwrap().value(), "class2");

            let values = &declaration.properties;
            {
                let value = values.properties.get(0).unwrap();
                assert!(value.attrs.is_empty());
                assert!(value.state.is_none());
                assert!(matches!(value.property, Expr::Call(_)));
            }
            {
                let value = values.properties.get(1).unwrap();
                {
                    let state = value.state.as_ref().unwrap();
                    if let PropertyStateAttValue::Single(name) = state.values.get(0).unwrap() {
                        assert_eq!(name, "active")
                    } else {
                        unreachable!()
                    }
                    if let PropertyStateAttValue::Parenthed {
                        parent,
                        paren: _,
                        inner_value,
                    } = state.values.get(1).unwrap()
                    {
                        assert_eq!(parent, "not");
                        assert_eq!(inner_value, "disabled");
                    } else {
                        unreachable!()
                    }
                }
                assert_eq!(value.attrs.len(), 1);

                assert_eq!(value.attrs[0].name, "clone");
                assert_eq!(
                    value.attrs[0].parenthesises_content.as_ref().unwrap()[0],
                    "some_val"
                );

                assert!(value.state.is_some());

                assert!(matches!(value.property, Expr::Closure(_)));
            }
        }

        {
            let declaration = &declarations.declarations[1];

            assert_eq!(declaration.classes.get_classes_string(), ["class2"]);

            let property = declaration.properties.properties.get(0).unwrap();

            assert!(property.attrs.is_empty());

            assert!(property.state.is_none());

            assert!(matches!(property.property, Expr::Path(_)));
        }
    }
}
