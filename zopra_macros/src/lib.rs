use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, FnArg, ItemFn, Pat, parse_macro_input};

// TODO: later on if depending on signals are present or not, impl RenderOnce else Render like we do now
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);

    let name = &function.sig.ident;
    let body = &function.block;

    let mut fields = Vec::new();
    let mut params = Vec::new();
    let mut values = Vec::new();
    let mut locals = Vec::new();

    for arg in &function.sig.inputs {
        let FnArg::Typed(arg) = arg else {
            return Error::new_spanned(arg, "component cannot have self")
                .to_compile_error()
                .into();
        };

        let Pat::Ident(pattern) = &*arg.pat else {
            return Error::new_spanned(&arg.pat, "component parameters must be identifiers")
                .to_compile_error()
                .into();
        };

        let name = &pattern.ident;
        let ty = &arg.ty;

        fields.push(quote! { #name: #ty });
        params.push(quote! { #name: #ty });
        values.push(quote! { #name });
        locals.push(quote! { let #name = &self.#name; });
    }

    let constructor = quote! {
        impl #name {
            pub fn new(#(#params),*) -> Self {
                Self {
                    #(#values),*
                }
            }
        }
    };

    quote! {
        struct #name {
            #(#fields),*
        }

        #constructor

        impl Render for #name {
            fn render(
                &mut self,
                cx: &mut ViewContext<Self>,
            ) -> impl IntoElement {
                #(#locals)*

                #body
            }
        }
    }
    .into()
}
