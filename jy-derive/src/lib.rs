use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Lit, Meta, NestedMeta};

#[proc_macro_derive(JyMenu, attributes(i18n))]
pub fn menu(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let variants = match &ast.data {
        Data::Enum(v) => &v.variants,
        _ => {
            return syn::Error::new(Span::call_site().into(), "Only enums are supported")
                .into_compile_error()
                .into();
        }
    };

    let mb_iter = variants
        .iter()
        .map(|variant| {
            variant.attrs.first().and_then(|attr| {
                let meta = attr.parse_meta().unwrap();
                match meta {
                    Meta::Path(_) => None,
                    Meta::List(v) => {
                        let inner = v.nested.first().unwrap();
                        if let NestedMeta::Lit(Lit::Int(lit)) = &*inner {
                            Some(lit.base10_parse::<i32>().unwrap())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
        })
        .map(|mb_i32| mb_i32.unwrap_or(0));

    let struct_name = &ast.ident;
    let idents = variants.iter().map(|v| &v.ident);
    let names = variants.iter().map(|v| v.ident.to_string());
    let len = names.len();
    let indices = 0..names.len();

    let idents_clone = idents.clone();
    let indices_clone = indices.clone();
    let to_idx = quote! {
        fn to_idx(&self) -> usize {
                match self {
                    #(#struct_name::#idents_clone {..} => #indices_clone,)*
                }
            }
    };

    let indices_clone = indices.clone();
    let idents_clone = idents.clone();
    let from = quote! {
        pub fn from(idx : usize) -> Self {
            match idx {
                #(#indices_clone => #struct_name::#idents_clone,)*
                _ => {
                    panic!("not here")
                }
            }
        }
    };

    let idents_clone = idents.clone();
    let i18n = quote! {
        fn i18n(&self) -> Option<i32> {
            let lang_id = match self {
                #(#struct_name::#idents_clone {..} => #mb_iter,)*
            };
            if lang_id > 0 {
                Some(lang_id)
            } else {
                None
            }
        }

    };

    let default = quote! {
        impl crate::game::Menu for #struct_name {
            fn up(&self) -> Self {
                if self.to_idx() == 0 {
                    self.clone()
                } else {
                    Self::from(self.to_idx() - 1)
                }
            }

            fn down(&self) -> Self {
                if self.to_idx() == Self::count() - 1 {
                    self.clone()
                } else {
                    Self::from(self.to_idx() + 1)
                }
            }

            fn to_name(&self) ->  String {
                let name = match self {
                    #(#struct_name::#idents {..} => #names,)*
                };
                name.into()
            }

            #to_idx

            #i18n
        }

        impl #struct_name {

            #from

            pub fn count() -> usize {
                #len
            }
        }
    };
    default.into()
}
