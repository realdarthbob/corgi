use std::any::TypeId;

use corgi_macros::rpc_fn;
use wincode::{SchemaRead, SchemaWrite};

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

struct Arg1(String);

impl<'de> SchemaRead<'de> for Arg1 {
    type Dst = Arg1;

    fn read(
        _: &mut impl wincode::io::Reader<'de>,
        _: &mut std::mem::MaybeUninit<Self::Dst>,
    ) -> wincode::ReadResult<()> {
        todo!()
    }
}

#[test]
fn rpc_fn_should_create_rpc_function_instance_with_custom_multiple_arguments_fn_with_custom_return_type()
 {
    struct Arg2(String);

    impl<'de> SchemaRead<'de> for Arg2 {
        type Dst = Arg2;

        fn read(
            _: &mut impl wincode::io::Reader<'de>,
            _: &mut std::mem::MaybeUninit<Self::Dst>,
        ) -> wincode::ReadResult<()> {
            todo!()
        }
    }

    struct ReturnType {
        _data: String,
        _data2: String,
    }

    impl SchemaWrite for ReturnType {
        type Src = ReturnType;

        fn size_of(_: &Self::Src) -> wincode::WriteResult<usize> {
            todo!()
        }

        fn write(_: &mut impl wincode::io::Writer, _: &Self::Src) -> wincode::WriteResult<()> {
            todo!()
        }
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

    let codec = corgi::codec::BincodeCodec;

    let args = (10_i32, 20_i32);
    let input_bytes = codec.encode(&args).unwrap();

    let handler = __CORGI_RPC_foo_multiple_args_return_type.handler.clone();
    let result_bytes = handler(input_bytes, codec.clone()).await.unwrap();

    let result: i32 = codec.decode(result_bytes).unwrap();
    assert_eq!(result, 30);
}
