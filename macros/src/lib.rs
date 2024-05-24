use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Message)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse_macro_input!(input);
    let name = &ast.ident;

    quote! {
        impl Message for #name {}
    }
    .into()
}
