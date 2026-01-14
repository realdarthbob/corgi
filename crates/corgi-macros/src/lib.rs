//! # Corgi Macros
//!
//! This crate provides procedural macros for the `corgi` RPC framework.
//! These macros are designed to work in tandem with the types defined in the
//! core `corgi` crate.
//!
//! ## Overview
//!
//! The primary macro is [`rpc_fn`], which automates the creation of RPC
//! metadata and execution handlers.
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{FnArg, ItemFn, ReturnType, parse_macro_input};

/// Marks an async function as an RPC-capable function.
///
/// This attribute does two things:
/// 1. It keeps the original function intact so it can be called locally.
/// 2. It generates a global `static` variable named `__CORGI_RPC_<fn_name>`
///    of type [`corgi::protocol::RpcFunction`].
///
/// # Requirements
/// - All arguments must implement `wincode::SchemaReadOwned`.
/// - The return type must implement `wincode::SchemaWrite`.
/// - The function must be `async`.
///
/// # Example
/// ```rust
/// #[rpc_fn]
/// async fn add(a: i32, b: i32) -> i32 {
///     a + b
/// }
///
/// // You can now access the metadata:
/// println!("RPC Name: {}", __CORGI_RPC_add.name);
/// ```
#[proc_macro_attribute]
pub fn rpc_fn(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    let fn_ident = &func.sig.ident;
    let fn_name_str = fn_ident.to_string();

    let rpc_ident = syn::Ident::new(&format!("__CORGI_RPC_{}", fn_ident), Span::call_site());

    let params: Vec<(syn::Ident, &syn::Type)> = func
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(pat) => {
                let ident = match &*pat.pat {
                    syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                    _ => panic!("rpc_fn only supports simple identifiers"),
                };
                (ident, &*pat.ty)
            }
            FnArg::Receiver(_) => panic!("rpc_fn does not support self"),
        })
        .collect();

    let param_descriptors = params.iter().map(|(ident, ty)| {
        let name_str = ident.to_string();
        quote! {
            corgi::protocol::Param {
                name: #name_str,
                type_id: std::any::TypeId::of::<#ty>(),
            }
        }
    });

    let param_types: Vec<_> = params.iter().map(|(_, ty)| ty).collect();
    let arg_idents: Vec<_> = params.iter().map(|(ident, _)| ident.clone()).collect();
    let tuple_type = quote! { ( #(#param_types),* ) };

    let has_return = match &func.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, _) => true,
    };

    let return_type_expr = if has_return {
        if let ReturnType::Type(_, ty) = &func.sig.output {
            quote! { Some(std::any::TypeId::of::<#ty>()) }
        } else {
            unreachable!()
        }
    } else {
        quote! { None }
    };

    let handler_body = if has_return {
        quote! {
            let result = #fn_ident( #(#arg_idents),* ).await;
            codec.encode(&result)
        }
    } else {
        quote! {
            #fn_ident( #(#arg_idents),* ).await;
            Ok(bytes::Bytes::new())
        }
    };

    let expanded = quote! {
        #func

        #[allow(non_upper_case_globals)]
        pub static #rpc_ident: std::sync::LazyLock<corgi::protocol::RpcFunction> =
        std::sync::LazyLock::new(|| {
            corgi::protocol::RpcFunction {
                name: #fn_name_str,
                params: vec![ #(#param_descriptors),* ],
                return_type: #return_type_expr,
                handler: std::sync::Arc::new(
                    |bytes: bytes::Bytes, codec: corgi::codec::BincodeCodec| {
                        use futures::FutureExt;

                        async move {
                            let args: #tuple_type = codec.decode(bytes)?;
                            let ( #(#arg_idents),* ) = args;
                            #handler_body
                        }.boxed()
                    }
                ),
            }
        });
    };

    expanded.into()
}
