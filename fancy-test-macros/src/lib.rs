extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};
use quote::quote;

#[proc_macro_attribute]
pub fn unit_test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let fn_name_ident = &input.sig.ident;
    let fn_name_str = syn::LitStr::new(&format!("::{}", fn_name_ident), proc_macro2::Span::call_site());
    let fn_body = &input.block;

    let expanded = quote!(
        #[allow(dead_code,non_upper_case_globals)]
        #[test_case]
        pub const #fn_name_ident: ::fancy_test::UnitTest = ::fancy_test::UnitTest {
            name: concat!(module_path!(), #fn_name_str),
            f: || {
                #fn_body
            }
        };
    );

    TokenStream::from(expanded)
}
