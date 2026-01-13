use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{FnArg, ItemFn, parse_macro_input};

#[proc_macro]
pub fn rpc_fn(input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);

    let fn_ident = &func.sig.ident;
    let fn_name_str = fn_ident.to_string();

    let param_types: Vec<_> = func
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(pat) => &pat.ty,
            FnArg::Receiver(_) => {
                panic!("rpc_fn does not support methods with self")
            }
        })
        .collect();

    let param_descriptors = param_types.iter().map(|ty| {
        quote! {
            corgi::protocol::Param {
                type_id: std::any::TypeId::of::<#ty>(),
            }
        }
    });

    let tuple_type = quote! {
        ( #(#param_types),* )
    };

    let arg_idents: Vec<_> = (0..param_types.len())
        .map(|i| syn::Ident::new(&format!("arg{}", i), Span::call_site()))
        .collect();

    let return_ty = match &func.sig.output {
        syn::ReturnType::Type(_, ty) => ty,
        syn::ReturnType::Default => &Box::new(syn::parse_quote!(())),
    };

    let expanded = quote! {{
        use bytes::Bytes;
        use futures::FutureExt;

        corgi::protocol::RpcFunction {
            name: #fn_name_str,
            params: vec![ #(#param_descriptors),* ],
            return_type: std::any::TypeId::of::<#return_ty>(),
            handler: std::sync::Arc::new(|bytes: Bytes, codec: corgi::codec::BincodeCodec| {
                async move {
                    let args: #tuple_type = codec.decode(bytes)?;
                    let ( #(#arg_idents),* ) = args;
                    let result = #fn_ident( #(#arg_idents),* ).await;
                    codec.encode(result)
                }
                .boxed()
            }),
        }
    }};

    expanded.into()
}
