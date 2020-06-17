use analyzer::IrVisitor;
use std::str::FromStr;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();

    if args[0] == "--single" {
        visit(&args[1]);
    } else {
        let files = std::fs::read_to_string(&args[0]).unwrap();
        for file_name in files.lines() {
            visit(file_name);
        }
    }
}

fn visit(file_name: &str) {
    println!("{:?}", file_name);
    let source = std::fs::read_to_string(file_name).unwrap();

    let file = if let Ok(file) = syn::parse_str(&source) {
        file
    } else {
        return;
    };

    let mut stream: std::collections::VecDeque<_> = proc_macro2::TokenStream::from_str(&source)
        .unwrap()
        .into_iter()
        .collect();

    while let Some(tree) = stream.pop_front() {
        let (ty_, span) = match tree {
            proc_macro2::TokenTree::Group(group) => {
                let span = group.span_open();
                let mut new: std::collections::VecDeque<_> = group.stream().into_iter().collect();
                new.extend(stream.into_iter());
                stream = new;
                // for tree in group.stream().into_iter() {
                //     group.extend()
                //     stream.
                // }
                ("group", span)
            }
            proc_macro2::TokenTree::Ident(ident) => ("ident", ident.span()),
            proc_macro2::TokenTree::Literal(lit) => ("lit", lit.span()),
            proc_macro2::TokenTree::Punct(punct) => ("punct", punct.span()),
        };
        // eprintln!("{} {:?} {:?}", ty_, span.start(), span.end());
    }

    let visitor = IrVisitor::new(file, source);
    let _ = visitor.visit();
}
