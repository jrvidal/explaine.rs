use std::collections::{HashMap, HashSet};
use syn_codegen::{Data, Definitions, Type};

const SYN: &str = include_str!("../syn.json");

fn main() {
    let defs: Definitions = serde_json::from_str(SYN).unwrap();

    let mut id_map = IdMap::default();
    let mut to_parents = HashMap::new();

    for node in &defs.types {
        let id = id_map.insert(&format!("syn::{}", node.ident));

        let mut types = Vec::new();

        match &node.data {
            Data::Struct(fields) => {
                types.extend(fields.values());
            }
            Data::Enum(variants) => {
                types.extend(variants.values().flat_map(|v| v.iter()));
            }
            Data::Private => {}
        }

        while let Some(ty) = types.pop() {
            match ty {
                Type::Syn(ident) => {
                    let field_id = id_map.insert(&format!("syn::{}", ident));
                    to_parents.entry(field_id).or_insert_with(Vec::new).push(id);
                }
                Type::Option(ty) => {
                    types.push(&*ty);
                }
                Type::Box(ty) => {
                    types.push(&*ty);
                }
                Type::Tuple(tuple_types) => {
                    types.extend(tuple_types.iter());
                }
                Type::Vec(ty) => {
                    types.push(&*ty);
                }
                Type::Token(token) => {
                    let token_id = id_map.insert(&format!("token::{}", token));
                    to_parents.entry(token_id).or_insert_with(Vec::new).push(id);
                }
                Type::Punctuated(punct) => {
                    types.push(&*punct.element);
                    let token_id = id_map.insert(&format!("token::{}", punct.punct));
                    to_parents.entry(token_id).or_insert_with(Vec::new).push(id);
                }
                Type::Ext(..) => {}
                Type::Group(..) => {}
                Type::Std(..) => {}
            }
        }
    }

    let mut args = std::env::args().skip(1);
    let maybe_ty = args.next();
    let depth = args
        .next()
        .map(|s| u8::from_str_radix(&s, 10).unwrap())
        .unwrap_or(1);

    let ty = if let Some(ty) = maybe_ty {
        ty
    } else {
        return print_all(to_parents, id_map, depth);
    };
    let id = id_map.name_to_id[&ty[..]];
    let mut queue = vec![(id, 0)];

    println!("digraph Deps {{\n");

    let mut visited = HashSet::new();

    while let Some((id, deep)) = queue.pop() {
        if deep >= depth {
            continue;
        }

        if !visited.insert(id) {
            continue;
        }

        if !to_parents.contains_key(&id) {
            continue;
        }

        let parents = &to_parents[&id];
        let name = &id_map.id_to_name[&id];
        for parent in parents {
            let parent_name = &id_map.id_to_name[parent];
            println!("\"{}\" -> \"{}\"", name, parent_name);
        }
        queue.extend(parents.iter().map(|p| (*p, deep + 1)));
    }

    println!("}}\n");
}

fn print_all(to_parents: HashMap<usize, Vec<usize>>, id_map: IdMap, max_depth: u8) {
    let mut to_children = HashMap::new();

    for &id in id_map.id_to_name.keys() {
        if let Some(parents) = to_parents.get(&id) {
            for parent in parents {
                to_children
                    .entry(parent)
                    .or_insert_with(HashSet::new)
                    .insert(id);
            }
        } else {
            eprintln!("no parents for {}", &id_map.id_to_name[&id]);
        }
    }

    eprintln!(
        "{:?}",
        to_children[&id_map.name_to_id["syn::Macro"]]
            .iter()
            .map(|id| &id_map.id_to_name[id])
            .collect::<Vec<_>>()
    );

    let mut queue = id_map
        .id_to_name
        .keys()
        .cloned()
        .filter(|id| !to_children.contains_key(&id))
        .map(|t| (t, 0))
        .collect::<Vec<_>>();

    eprintln!(
        "{:?}",
        queue
            .iter()
            .map(|id| &id_map.id_to_name[&id.0])
            .collect::<Vec<_>>()
    );
    println!("digraph Deps {{\n");

    while let Some((id, depth)) = queue.pop() {
        if depth >= max_depth {
            continue;
        }

        if !to_parents.contains_key(&id) {
            eprintln!("discarding {}", &id_map.id_to_name[&id]);
            continue;
        }

        let parents = &to_parents[&id];

        let name = &id_map.id_to_name[&id];

        if name == "syn::Macro" {
            panic!("{:?}", depth);
        }

        for parent in parents {
            let parent_name = &id_map.id_to_name[parent];
            println!("{:?} -> {:?}", name, parent_name);
        }

        queue.extend(parents.iter().map(|&p| (p, depth + 1)));
    }

    println!("}}\n");
}

#[derive(Default)]
struct IdMap {
    name_to_id: HashMap<String, usize>,
    id_to_name: HashMap<usize, String>,
    base_id: usize,
}

impl IdMap {
    fn insert(&mut self, name: &str) -> usize {
        let base = &mut self.base_id;
        let id_ref = self.name_to_id.entry(name.to_string()).or_insert_with(|| {
            let next = *base;
            *base += 1;
            next
        });

        self.id_to_name.entry(*id_ref).or_insert(name.to_string());

        *id_ref
    }
}