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
case![bound_lifetimes];
case![enums];
case![fn_type];
case![inner_doc_comment];
case![item_use_special];
case![let_stmt];
case![loops];
case![nested_item_comment];
case![paths];
case![qself];
case![raw_ident];
case![receiver];
case![macros];
case![type_reference];
case![type_array];
case![let_patterns];
case![struct_field];
case![tuple_struct_pat];
case![unit];
case![visibility];
