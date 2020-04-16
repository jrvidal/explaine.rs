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
                    to_parents
                        .entry(field_id)
                        .or_insert_with(HashSet::new)
                        .insert(id);
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
                    to_parents
                        .entry(token_id)
                        .or_insert_with(HashSet::new)
                        .insert(id);
                }
                Type::Punctuated(punct) => {
                    types.push(&*punct.element);
                    let token_id = id_map.insert(&format!("token::{}", punct.punct));
                    to_parents
                        .entry(token_id)
                        .or_insert_with(HashSet::new)
                        .insert(id);
                }
                Type::Ext(..) => {}
                Type::Group(..) => {}
                Type::Std(..) => {}
            }
        }
    }

    let mut to_children = HashMap::new();

    for &id in id_map.id_to_name.keys() {
        if let Some(parents) = to_parents.get(&id) {
            for parent in parents {
                to_children
                    .entry(*parent)
                    .or_insert_with(HashSet::new)
                    .insert(id);
            }
        }
    }

    let to_children = to_children;

    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    let child_to_parent = if args[0] == "--reverse" {
        let _ = args.remove(0);
        false
    } else {
        true
    };

    let no_tokens = if args[0] == "--no-tokens" {
        let _ = args.remove(0);
        true
    } else {
        false
    };

    enum Accept {
        Type,
        Depth,
        Blacklist,
    }

    let mut types = vec![];
    let mut accepting = Accept::Type;
    let mut blacklist = HashSet::new();

    for arg in args {
        match accepting {
            Accept::Type => {
                if arg == "--blacklist" {
                    accepting = Accept::Blacklist;
                } else {
                    accepting = Accept::Depth;
                    let id = id_map.name_to_id[&arg];
                    types.push((id, 1, 0));
                }
            }
            Accept::Blacklist => {
                accepting = Accept::Type;
                let id = id_map.name_to_id[&arg];
                blacklist.insert(id);
            }
            Accept::Depth => {
                types.last_mut().unwrap().1 = u8::from_str_radix(&arg, 10).unwrap();
                accepting = Accept::Type;
            }
        }
    }

    println!("digraph Deps {{\n");

    let mut visited = HashSet::new();

    let nodes = if child_to_parent {
        &to_parents
    } else {
        &to_children
    };

    while let Some((id, max_depth, depth)) = types.pop() {
        if depth >= max_depth {
            continue;
        }

        if !visited.insert(id) {
            continue;
        }

        if !nodes.contains_key(&id) {
            continue;
        }

        if blacklist.contains(&id) {
            continue;
        }

        let name = &id_map.id_to_name[&id];

        if no_tokens && name.starts_with("token::") {
            continue;
        }

        let neighbours = &nodes[&id];

        for neighbour in neighbours {
            let neighbour_name = &id_map.id_to_name[neighbour];
            if no_tokens && neighbour_name.starts_with("token::") {
                continue;
            }
            println!("{:?} -> {:?}", name, neighbour_name);
        }

        types.extend(neighbours.iter().map(|p| (*p, max_depth, depth + 1)));
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
