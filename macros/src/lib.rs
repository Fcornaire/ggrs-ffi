use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn catch_status(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
                    Ok(_) => Status::ok(),
                    Err(e) => {
                        Status::ko(Box::leak(e.into_boxed_str()))
                    }
                }
            }) {
                Ok(status) => status,
                Err(e) => {
                    let error_msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    }
                    else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    }
                    else {
                        "unknown error".to_string()
                    };
                    Status::ko(Box::leak(error_msg.into_boxed_str()))
                }
            }
        }
    };

    let output = TokenStream::from(output);
    output
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
