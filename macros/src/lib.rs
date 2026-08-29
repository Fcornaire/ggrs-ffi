use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn catch_status(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let vis = input.vis.clone();
    let sig = input.sig.clone();
    let attrs = input.attrs.clone();
    let ident = sig.ident.clone();
    let unsafety = sig.unsafety;
    let abi = sig.abi.clone();
    let generics = sig.generics.clone();
    let inputs = sig.inputs.clone();
    let output = sig.output.clone();
    let body = input.block.clone();

    let abi_tokens = if let Some(a) = abi {
        quote::quote! { #a }
    } else {
        quote::quote! { extern "C" }
    };

    let output = quote::quote! {
        #(#attrs)*
        #[no_mangle]
        #vis #unsafety #abi_tokens fn #ident #generics (#inputs) #output {
            match std::panic::catch_unwind(|| {
                match #body {
                    Ok(_) => Status::ok(),
                    Err(e) => Status::ko(&e),
                }
            }) {
                Ok(status) => status,
                Err(e) => {
                    let error_msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown error".to_string()
                    };
                    Status::ko(&error_msg)
                }
            }
        }
    };

    TokenStream::from(output)
}

#[proc_macro_attribute]
pub fn catch_action_result(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = input.sig.ident.clone();
    let fn_body = input.block.clone();
    let fn_return_type = &input.sig.output;
    let fn_args = &input.sig.inputs;

    let output = quote! {
        #[no_mangle]
        unsafe fn #fn_name(#fn_args) #fn_return_type {
            match std::panic::catch_unwind(|| {
                match #fn_body {
                    Ok(sb) => ActionResult::ok(sb),
                    Err(e) => {
                        ActionResult::ko(e, UnmanagedBytes::empty().to_safe_bytes())
                    }
                }
            }) {
                Ok(action_result) => action_result,
                Err(e) => {
                    if let Some(er) = e.downcast_ref::<&str>() {
                        return ActionResult::ko(er.to_string(), UnmanagedBytes::empty().to_safe_bytes());
                    }
                    else if let Some(er) = e.downcast_ref::<String>() {
                        return ActionResult::ko(er.to_string(), UnmanagedBytes::empty().to_safe_bytes());
                    }
                    else {
                        return ActionResult::ko("Unkown error".to_string(),UnmanagedBytes::empty().to_safe_bytes());
                    }
                }
            }
        }
    };

    let output = TokenStream::from(output);
    output
}
