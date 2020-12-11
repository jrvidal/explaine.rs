use std::fs;

mod runtime;

macro_rules! case {
    ($name:ident) => {
        #[test]
        fn $name() {
            let source =
                fs::read_to_string(&format!("src/tests/fixtures/{}.rs", stringify!($name)))
                    .expect("no file");
            runtime::test_example(&source);
        }
    };
}

case![arrays];
case![attributes];
case![binding_patterns];
case![bound_lifetimes];
case![comments];
case![enums];
case![extern_crate];
case![fn_type];
case![generics];
case![inner_doc_comment];
case![item_use];
case![let_patterns];
case![let_stmt];
case![loops];
case![macros];
case![nested_item_comment];
case![paths];
case![qself];
case![raw_ident];
case![receiver];
case![returns];
case![struct_field];
case![tuple_struct_pat];
case![type_array];
case![type_reference];
case![unit];
case![visibility];
