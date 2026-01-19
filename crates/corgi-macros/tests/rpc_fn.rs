use std::any::TypeId;

use corgi_macros::rpc_fn;
use prost::Message;

#[test]
fn rpc_fn_should_create_rpc_function_instance_with_zero_arguments_fn() {
    #[rpc_fn]
    async fn foo_empty_args() {}

    assert_eq!(__CORGI_RPC_foo_empty_args.name, "foo_empty_args");
    assert!(__CORGI_RPC_foo_empty_args.params.is_empty());
    assert!(__CORGI_RPC_foo_empty_args.return_type.is_none());
}

#[test]
fn rpc_fn_should_create_rpc_function_instance_with_one_arguments_fn() {
    #[rpc_fn]
    async fn foo_one_arg(arg: i32) {
        println!("{arg}");
    }

    assert_eq!(__CORGI_RPC_foo_one_arg.name, "foo_one_arg");
    assert!(!__CORGI_RPC_foo_one_arg.params.is_empty());
    assert_eq!(__CORGI_RPC_foo_one_arg.params[0].name, "arg",);
    assert_eq!(
        __CORGI_RPC_foo_one_arg.params[0].type_id,
        TypeId::of::<i32>()
    );
    assert!(__CORGI_RPC_foo_one_arg.return_type.is_none());
}

#[test]
fn rpc_fn_should_create_rpc_function_instance_with_multiple_arguments_fn() {
    #[rpc_fn]
    async fn foo_multiple_args(arg1: i32, arg2: i32) {
        println!("{arg1}-{arg2}");
    }

    assert_eq!(__CORGI_RPC_foo_multiple_args.name, "foo_multiple_args");
    assert!(!__CORGI_RPC_foo_multiple_args.params.is_empty());
    assert_eq!(__CORGI_RPC_foo_multiple_args.params[0].name, "arg1");
    assert_eq!(
        __CORGI_RPC_foo_multiple_args.params[0].type_id,
        TypeId::of::<i32>()
    );

    assert_eq!(__CORGI_RPC_foo_multiple_args.params[1].name, "arg2");
    assert_eq!(
        __CORGI_RPC_foo_multiple_args.params[1].type_id,
        TypeId::of::<i32>()
    );
    assert!(__CORGI_RPC_foo_multiple_args.return_type.is_none());
}

#[test]
fn rpc_fn_should_create_rpc_function_instance_with_multiple_arguments_fn_with_return_type() {
    #[rpc_fn]
    async fn foo_multiple_args_return_type(arg1: i32, arg2: i32) -> i32 {
        arg1 + arg2
    }

    assert_eq!(
        __CORGI_RPC_foo_multiple_args_return_type.name,
        "foo_multiple_args_return_type"
    );
    assert!(!__CORGI_RPC_foo_multiple_args_return_type.params.is_empty());
    assert_eq!(
        __CORGI_RPC_foo_multiple_args_return_type.params[0].name,
        "arg1"
    );
    assert_eq!(
        __CORGI_RPC_foo_multiple_args_return_type.params[0].type_id,
        TypeId::of::<i32>()
    );

    assert_eq!(
        __CORGI_RPC_foo_multiple_args_return_type.params[1].name,
        "arg2"
    );
    assert_eq!(
        __CORGI_RPC_foo_multiple_args_return_type.params[1].type_id,
        TypeId::of::<i32>()
    );
    assert_eq!(
        __CORGI_RPC_foo_multiple_args_return_type.return_type,
        Some(TypeId::of::<i32>())
    );
}

#[derive(Message, Clone, PartialEq)]
struct Arg1(#[prost(string, tag = "1")] String);

#[test]
fn rpc_fn_should_create_rpc_function_instance_with_custom_multiple_arguments_fn_with_custom_return_type()
 {
    #[derive(Message, Clone, PartialEq)]
    struct Arg2(#[prost(string, tag = "1")] String);

    #[derive(Message, Clone, PartialEq)]
    struct ReturnType {
        #[prost(string, tag = "1")]
        _data: String,
        #[prost(string, tag = "2")]
        _data2: String,
    }

    #[rpc_fn]
    async fn foo_custom_multiple_args_custom_return_type(arg1: Arg1, arg2: Arg2) -> ReturnType {
        ReturnType {
            _data: arg1.0,
            _data2: arg2.0,
        }
    }

    assert_eq!(
        __CORGI_RPC_foo_custom_multiple_args_custom_return_type.name,
        "foo_custom_multiple_args_custom_return_type"
    );
    assert!(
        !__CORGI_RPC_foo_custom_multiple_args_custom_return_type
            .params
            .is_empty()
    );
    assert_eq!(
        __CORGI_RPC_foo_custom_multiple_args_custom_return_type.params[0].name,
        "arg1"
    );
    assert_eq!(
        __CORGI_RPC_foo_custom_multiple_args_custom_return_type.params[0].type_id,
        TypeId::of::<Arg1>()
    );

    assert_eq!(
        __CORGI_RPC_foo_custom_multiple_args_custom_return_type.params[1].name,
        "arg2"
    );
    assert_eq!(
        __CORGI_RPC_foo_custom_multiple_args_custom_return_type.params[1].type_id,
        TypeId::of::<Arg2>()
    );
    assert_eq!(
        __CORGI_RPC_foo_custom_multiple_args_custom_return_type.return_type,
        Some(TypeId::of::<ReturnType>())
    );
}

#[tokio::test]
async fn test_rpc_execution() {
    #[rpc_fn]
    async fn foo_multiple_args_return_type(arg1: i32, arg2: i32) -> i32 {
        arg1 + arg2
    }

    let codec = corgi::protocol::codec::ProtobufCodec;

    let args = vec![
        codec.encode(&10_i32).unwrap(),
        codec.encode(&20_i32).unwrap(),
    ];

    let handler = __CORGI_RPC_foo_multiple_args_return_type.handler.clone();
    let result_bytes = handler(args, codec.clone()).await.unwrap();

    let result: i32 = codec.decode(&result_bytes).unwrap();
    assert_eq!(result, 30);
}
