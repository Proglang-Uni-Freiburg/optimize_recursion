mod ast;

use proc_macro;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn;
use syn::parse_macro_input;
use crate::ast::OptimizableFunction;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn optimize_recursion(_attr: proc_macro::TokenStream, tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_function: syn::ItemFn = parse_macro_input!(tokens as syn::ItemFn);
    let function = OptimizableFunction::from(input_function.clone());
    let optimized_result = function.optimize();
    let result: proc_macro::TokenStream = quote!(#optimized_result).into();
    println!("result function: {}", result);
    return result;

    // TODO: use result
    // let result = quote!(#input_function).into();
    // result
}