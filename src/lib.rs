//! [<img alt="Static Badge" src="https://img.shields.io/badge/github-semiwaker%2Fttgraph-blue?style=for-the-badge&logo=github">](https://github.com/semiwaker/TTGraph)
//! [<img alt="Static Badge" src="https://img.shields.io/badge/crates.io-ttgraph-orange?style=for-the-badge&logo=rust">](https://crates.io/crates/ttgraph)
//! [<img alt="Static Badge" src="https://img.shields.io/badge/dos.rs-ttgraph-green?style=for-the-badge&logo=docs.rs">](https://docs.rs/ttgraph)
//!
//! TTGraph is:
//! + A container or database for many different data, which cross-reference each other, forming a graph-like data structure.
//! + **Typed graph:** A collection of multiple types of nodes. Each node hold some private data, and some pointers/edges/references to other nodes.
//! + **Transactional graph:** All operations on the graph are organized by transaction, which means an atomic group of operation is applied at the same time.
//!
//! TTGraph provides:
//! + A convinient container for different types of data, which provides some useful methods to deal with types.
//! + A data struct to maintain the connection between nodes. TTGraph create a reflection for all types to track the connection between nodes, named as *link*. This allows some fancy operations, such as redirect links and maintain bidirectional links.
//! + A clean interface to help get rid of some annoying compile errors. The design of transaction tries to prevent having a non-mutable reference and a mutable reference of the same object at the same time, and tries not to get into a maze of lifetimes.
//! + TTGraph is originally designed as an Intermediate Representation system for compilers, but its potential is not limited.
//!
//! TTGraph does **not** currently provides, but may be improved in the future:
//! + Very high performance. Though TTGraph operations are relatively cheap (mostly O(log(n))), it is not a high performance database.
//! + Very large capacity. All data are stored in memory.
//!
//! # Motivational Example
//!
//! ## Typed Node Declaration
//!
//! Assume there are a few factories, workers and products, the following example use TTGraph to maintain their data.
//!
//! ```
//! use ttgraph::*;
//! use std::collections::HashSet;
//!
//! #[derive(TypedNode)]
//! struct FactoryNode{
//!   name: String,
//!   workers: HashSet<NodeIndex>,
//!   products: HashSet<NodeIndex>,
//! }
//!
//! #[derive(TypedNode)]
//! struct WorkerNode{
//!   name: String,
//!   factory: NodeIndex,
//!   produced: Vec<NodeIndex>,
//! }
//!
//! #[derive(TypedNode)]
//! struct ProductNode{
//!   id: usize
//! }
//! ```
//!
//! Here, a factory have a name, multiple workers and products. `name` is a **data field**, which TTGraph does not care about. It can be any type in Rust.
//!
//! `workers` and `products` are **links**. A link is a connection to another node. TTGraph use [`NodeIndex`] to index a node, which implements [`Copy`]. If field is one of the following types, it is treated as a link. (Note: types are matched by name in the macros, `ttgraph::NodeIndex`/`NodeIndex`/`std::collections::Vec::<NodeIndex>`/`Vec::<ttgraph::NodeIndex>` are all acceptable.)
//!
//! + Direct link: `NodeIndex`
//! + Vector link: `Vec<NodeIndex>`
//! + Unordered set link: `HashSet<NodeIndex>`
//! + Ordered set link: `BTreeSet<NodeIndex>`
//!
//! ## Graph and Transaction
//!
//! Next example shows how to build a graph.
//!
//! ```
//! # use ttgraph::*;
//! # use std::collections::HashSet;
//! # #[derive(TypedNode)]
//! # struct FactoryNode{
//! #  name: String,
//! #  workers: HashSet<NodeIndex>,
//! #  products: HashSet<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct WorkerNode{
//! #   name: String,
//! #   factory: NodeIndex,
//! #   produced: Vec<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct ProductNode{
//! #   id: usize
//! #  }
//! // Use an node_enum to collect all node types together
//! node_enum!{
//!   enum Node{
//!     Factory(FactoryNode),
//!     Worker(WorkerNode),
//!     Product(ProductNode),
//!   }
//! }
//!
//! # fn main() {
//! // Create the context
//! let ctx = Context::new();
//! // Create a graph of Node
//! let mut graph = Graph::<Node>::new(&ctx);
//!
//! // Does some initial operations with a transaction
//! // Actual type: Transaction::<Node>, <Node> can be inferenced when commited
//! let mut trans = Transaction::new(&ctx);
//! let product1 = trans.insert(Node::Product(ProductNode{ id: 1 }));
//! let product2 = trans.insert(Node::Product(ProductNode{ id: 2 }));
//! let worker1 = trans.alloc();
//! let worker2 = trans.alloc();
//! let factory = trans.insert(Node::Factory(FactoryNode{
//!   name: "Factory".to_string(),
//!   workers: HashSet::from([worker1, worker2]),
//!   products: HashSet::from([product1, product2]),
//! }));
//! trans.fill_back(worker1, Node::Worker(WorkerNode{
//!   name: "Alice".to_string(),
//!   factory,
//!   produced: vec![product2],
//! }));
//! trans.fill_back(worker2, Node::Worker(WorkerNode{
//!   name: "Bob".to_string(),
//!   factory,
//!   produced: vec![product1],
//! }));
//!
//! // Commit the transaction to the graph
//! graph.commit(trans);
//!
//! // Get the factory node back
//! let factory_node = get_node!(graph, Node::Factory, factory).unwrap();
//! assert_eq!(factory_node.name, "Factory");
//! assert_eq!(factory_node.workers, HashSet::from([worker1, worker2]));
//! assert_eq!(factory_node.products, HashSet::from([product1, product2]));
//! # }
//! ```
//!
//! First, the [`node_enum!`] macro is used to create a enum to collect all types of nodes. It is a proc_macro instead of proc_macro_derive for extendable syntax in the latter examples. The enum inside of `node_enum!` will implements trait `NodeEnum` and can be used in `Graph`.
//!
//! ```rust
//! # use ttgraph::*;
//! # use std::collections::HashSet;
//! # #[derive(TypedNode)]
//! # struct FactoryNode{
//! #  name: String,
//! #  workers: HashSet<NodeIndex>,
//! #  products: HashSet<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct WorkerNode{
//! #   name: String,
//! #   factory: NodeIndex,
//! #   produced: Vec<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct ProductNode{
//! #   id: usize
//! #  }
//! node_enum!{
//!   enum Node{
//!     Factory(FactoryNode),
//!     Worker(WorkerNode),
//!     Product(ProductNode),
//!   }
//! }
//! # fn main() {}
//! ```
//!
//! Then, create a [`Context`] and a [`Graph`] using that context. The context is used to ensure the NodeIndexes are consistent across all transactions. Graph does not hold a reference to the context, so it is the user's reponsibility to keep it.
//!
//! ```rust
//! # use ttgraph::*;
//! # use std::collections::HashSet;
//! # #[derive(TypedNode)]
//! # struct FactoryNode{
//! #  name: String,
//! #  workers: HashSet<NodeIndex>,
//! #  products: HashSet<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct WorkerNode{
//! #   name: String,
//! #   factory: NodeIndex,
//! #   produced: Vec<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct ProductNode{
//! #   id: usize
//! #  }
//! # node_enum!{
//! #   enum Node{
//! #     Factory(FactoryNode),
//! #     Worker(WorkerNode),
//! #     Product(ProductNode),
//! #   }
//! # }
//! # fn main() {
//! let ctx = Context::new();
//! let mut graph = Graph::<Node>::new(&ctx);
//! # }
//! ```
//!
//! Next, a [`Transaction`] is created using the same context as the graph. After operations are done on the transcations, it can be committed to the graph with method [`commit`](Graph::commit). Transaction does not hold a reference to the graph and they have independent lifetime. (Though, it does nothing if a transaction outlives the graph)
//!
//! ```rust
//! # use ttgraph::*;
//! # use std::collections::HashSet;
//! # #[derive(TypedNode)]
//! # struct FactoryNode{
//! #  name: String,
//! #  workers: HashSet<NodeIndex>,
//! #  products: HashSet<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct WorkerNode{
//! #   name: String,
//! #   factory: NodeIndex,
//! #   produced: Vec<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct ProductNode{
//! #   id: usize
//! #  }
//! # node_enum!{
//! #   enum Node{
//! #     Factory(FactoryNode),
//! #     Worker(WorkerNode),
//! #     Product(ProductNode),
//! #   }
//! # }
//! # fn main() {
//! # let ctx = Context::new();
//! # let mut graph = Graph::<Node>::new(&ctx);
//! let mut trans = Transaction::new(&ctx);
//! // Do something with trans
//! graph.commit(trans);
//! # }
//! ```
//!
//! Now we take a closer look on how to build the graph. `Product` nodes are the simplest, it only have a id. Use [`insert`](Transaction::insert) to add a node into the transaction. It returns a [`NodeIndex`] pointing to the new node, which means later we can use `product1` and `product2` to retrieve the node from the graph.
//!
//! ```rust
//! # use ttgraph::*;
//! # use std::collections::HashSet;
//! # #[derive(TypedNode)]
//! # struct FactoryNode{
//! #  name: String,
//! #  workers: HashSet<NodeIndex>,
//! #  products: HashSet<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct WorkerNode{
//! #   name: String,
//! #   factory: NodeIndex,
//! #   produced: Vec<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct ProductNode{
//! #   id: usize
//! #  }
//! # node_enum!{
//! #   enum Node{
//! #     Factory(FactoryNode),
//! #     Worker(WorkerNode),
//! #     Product(ProductNode),
//! #   }
//! # }
//! # fn main() {
//! # let ctx = Context::new();
//! # let mut graph = Graph::<Node>::new(&ctx);
//! # let mut trans = Transaction::new(&ctx);
//! let product1 = trans.insert(Node::Product(ProductNode{ id: 1 }));
//! let product2 = trans.insert(Node::Product(ProductNode{ id: 2 }));
//! # graph.commit(trans);
//! # }
//! ```
//!
//! Factories and workers have a more complex relationship, as they cross-refenerence each other. That means we cannot make a `FactoryNode` or a `WorkerNode` alone. Lucky, TTGraph does operations in transaction, we can first allocate a [`NodeIndex`] for the workers with method [`alloc`](Transaction::alloc), then fill the data back with method [`fill_back`](Transaction::fill_back). The transaction prevents dangling [`NodeIndex`] by checking all allocated nodes are filled back when committed.
//!
//! ```rust
//! # use ttgraph::*;
//! # use std::collections::HashSet;
//! # #[derive(TypedNode)]
//! # struct FactoryNode{
//! #  name: String,
//! #  workers: HashSet<NodeIndex>,
//! #  products: HashSet<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct WorkerNode{
//! #   name: String,
//! #   factory: NodeIndex,
//! #   produced: Vec<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct ProductNode{
//! #   id: usize
//! #  }
//! # node_enum!{
//! #   enum Node{
//! #     Factory(FactoryNode),
//! #     Worker(WorkerNode),
//! #     Product(ProductNode),
//! #   }
//! # }
//! # fn main() {
//! # let ctx = Context::new();
//! # let mut graph = Graph::<Node>::new(&ctx);
//! # let mut trans = Transaction::new(&ctx);
//! # let product1 = trans.insert(Node::Product(ProductNode{ id: 1 }));
//! # let product2 = trans.insert(Node::Product(ProductNode{ id: 2 }));
//! let worker1 = trans.alloc();
//! let worker2 = trans.alloc();
//! let factory = trans.insert(Node::Factory(FactoryNode{
//!   name: "Factory".to_string(),
//!   workers: HashSet::from([worker1, worker2]),
//!   products: HashSet::from([product1, product2]),
//! }));
//! trans.fill_back(worker1, Node::Worker(WorkerNode{
//!   name: "Alice".to_string(),
//!   factory,
//!   produced: vec![product2],
//! }));
//! trans.fill_back(worker2, Node::Worker(WorkerNode{
//!   name: "Bob".to_string(),
//!   factory,
//!   produced: vec![product1],
//! }));
//! # graph.commit(trans);
//! # }
//! ```
//!
//! Finally, after committing the transaction to the graph, we have a graph with the nodes described above. We can use [`NodeIndex`] to get the node back. [`get_node!`] macro is used when the type of the node is previously known, which returns an `Option<&TypedNode>` to indicate if the node is avaiable.
//! ```rust
//! # use ttgraph::*;
//! # use std::collections::HashSet;
//! # #[derive(TypedNode)]
//! # struct FactoryNode{
//! #  name: String,
//! #  workers: HashSet<NodeIndex>,
//! #  products: HashSet<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct WorkerNode{
//! #   name: String,
//! #   factory: NodeIndex,
//! #   produced: Vec<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct ProductNode{
//! #   id: usize
//! #  }
//! # node_enum!{
//! #   enum Node{
//! #     Factory(FactoryNode),
//! #     Worker(WorkerNode),
//! #     Product(ProductNode),
//! #   }
//! # }
//! # fn main() {
//! # let ctx = Context::new();
//! # let mut graph = Graph::<Node>::new(&ctx);
//! # let mut trans = Transaction::new(&ctx);
//! # let product1 = trans.insert(Node::Product(ProductNode{ id: 1 }));
//! # let product2 = trans.insert(Node::Product(ProductNode{ id: 2 }));
//! # let worker1 = trans.alloc();
//! # let worker2 = trans.alloc();
//! # let factory = trans.insert(Node::Factory(FactoryNode{
//! #   name: "Factory".to_string(),
//! #   workers: HashSet::from([worker1, worker2]),
//! #   products: HashSet::from([product1, product2]),
//! # }));
//! # trans.fill_back(worker1, Node::Worker(WorkerNode{
//! #   name: "Alice".to_string(),
//! #   factory,
//! #   produced: vec![product2],
//! # }));
//! # trans.fill_back(worker2, Node::Worker(WorkerNode{
//! #   name: "Bob".to_string(),
//! #   factory,
//! #   produced: vec![product1],
//! # }));
//! # graph.commit(trans);
//! let factory_node = get_node!(graph, Node::Factory, factory).unwrap();
//! assert_eq!(factory_node.name, "Factory");
//! assert_eq!(factory_node.workers, HashSet::from([worker1, worker2]));
//! assert_eq!(factory_node.products, HashSet::from([product1, product2]));
//! # }
//! ```
//!
//! For more operations, please view the documents on struct [`Graph`] and [`Transaction`].
//!
//! ## Bidiretional links
//!
//! TTGraph supports bidirectional link declaration. In this example, the `workers` field of `Factory` and the `factory` field of `Worker` is in fact a pair of bidirectional link. We can modify the [`node_enum!`] declaration for more supports.
//!
//! ```rust
//! # use ttgraph::*;
//! # use std::collections::HashSet;
//! # #[derive(TypedNode)]
//! # struct FactoryNode{
//! #  name: String,
//! #  workers: HashSet<NodeIndex>,
//! #  products: HashSet<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct WorkerNode{
//! #   name: String,
//! #   factory: NodeIndex,
//! #   produced: Vec<NodeIndex>,
//! # }
//! # #[derive(TypedNode)]
//! # struct ProductNode{
//! #   id: usize
//! #  }
//! node_enum!{
//!   enum Node{
//!     Factory(FactoryNode),
//!     Worker(WorkerNode),
//!     Product(ProductNode),
//!   }
//!   bidirectional!{
//!     Factory.workers <-> Worker.factory,
//!   }
//! }
//!
//! # fn main() {
//! let ctx = Context::new();
//! let mut graph = Graph::<Node>::new(&ctx);
//!
//! let mut trans = Transaction::new(&ctx);
//! let product1 = trans.insert(Node::Product(ProductNode{ id: 1 }));
//! let product2 = trans.insert(Node::Product(ProductNode{ id: 2 }));
//! let factory = trans.insert(Node::Factory(FactoryNode{
//!   name: "Factory".to_string(),
//!   // Here we leave this set empty to demonstrate it can be automatically filled
//!   workers: HashSet::new(),
//!   products: HashSet::from([product1, product2]),
//! }));
//! let worker1 = trans.insert(Node::Worker(WorkerNode{
//!   name: "Alice".to_string(),
//!   factory,
//!   produced: vec![product2],
//! }));
//! let worker2 = trans.insert(Node::Worker(WorkerNode{
//!   name: "Bob".to_string(),
//!   factory,
//!   produced: vec![product1],
//! }));
//!
//! graph.commit(trans);
//!
//! // Get the factory node back
//! let factory_node = get_node!(graph, Node::Factory, factory).unwrap();
//! assert_eq!(factory_node.name, "Factory");
//! assert_eq!(factory_node.workers, HashSet::from([worker1, worker2]));
//! assert_eq!(factory_node.products, HashSet::from([product1, product2]));
//! # }
//! ```
//!
//! Here, the `bidiretional!` macro inside of [`node_enum!`] macro is used to declare bidirecitonal links.
//!
//! + Use `variant.field <-> variant.field,` to indicate a pair of bidirecitonal links. Note: variant of the enum, not type!
//! + `bidiretional!` is not actually a macro, it can only be used inside of `node_enum!`
//!
//! Next, when making the factory node, its workers are simply left empty. However, after commited to the graph, TTGraph automatically adds the bidirectional links into it.
//!
//! Rules of bidiretional links are:
//!
//! + Bidirectional links may be formed between: a pair of `NodeIndex`, between `NodeIndex` and `Set<NodeIndex>`, a pair of `Set<NodeIndex>`. (`Set` may be `HashSet` or `BTreeSet`, `Vec` is not supported currently)
//! + When a link is added, the opposite side of the bidiretional link is checked. If the bidiretional link is already there, nothing happens. If that link have a place to be added, it is automatially added. Otherwise, it panics for conflict.
//! + When a link is removed, the opposite side of the bidiretional link is checked. If the bidiretional link is there, it is removed. Otherwise, since TTGraph does not know if the user removes it on purpose, it is assumed that nothing should happen.
//! + `NodeIndex` field: link can be added if it is [`NodeIndex::empty()`], otherwise it conflicts and panics. Link can be removed if it is not empty, but does not panic if it is.
//! + `Set<NodeIndex>` field: link can always be added into or removed from the set.
//! + When modifying existing pairs of bidiretional links, ensure the modification happens in the same transaction to prevent conflict. TTGraph does all other operations before maintaining bidiretional links.
//!
//! ## Get data by name and group
//!
//! TTGraph supports few operations for type erasure, targeting cases that some typed nodes have some similar fields, and matching the enum for these field is verbose.
//!
//! Following last example, assume there are two types of workers, robots and humans. They may have very different data, but they both have a name. Now we want to make a name list for all the workers. Typical solution is to match the NodeEnum, but TTGraph gives another solution by getting data by name.
//!
//! [`data_ref_by_name`](NodeEnum::data_ref_by_name) method provides an interface to access a data field by its name and type. If the node have that field and the type matches (through `std::any::Any::downcast_ref`), `Some(&Type)` is returned, otherwise `None` is returned.
//!
//! ```rust
//! # use ttgraph::*;
//! #[derive(TypedNode)]
//! struct HumanWorkerNode{
//!   name: String,
//!   // ... other data
//! }
//! #[derive(TypedNode)]
//! struct RobotWorkerNode{
//!   name: String,
//!   // ... other data
//! }
//!
//! node_enum!{
//!   enum Node{
//!     Human(HumanWorkerNode),
//!     Robot(RobotWorkerNode),
//!     // ... other nodes
//!   }
//! }
//!
//! # fn main() {
//! let ctx = Context::new();
//! let mut graph = Graph::<Node>::new(&ctx);
//! # let mut trans = Transaction::new(&ctx);
//! # let idx = trans.insert(Node::Human(HumanWorkerNode{name: "human".to_string()}));
//! # graph.commit(trans);
//! // ... building the graph
//!
//! // idx: NodeIndex, node: &Node
//! let node = graph.get(idx).unwrap();
//!
//! // Not so convinient way to get the name
//! let name = match node {
//!   Node::Human(human) => Some(&human.name),
//!   Node::Robot(robot) => Some(&robot.name),
//!   _ => None,
//! };
//!
//! // A simplified solution
//! // Here, "name" is the field's name
//! // The "name" field is a String, so this variable is an Option<&str>
//! let name = node.data_ref_by_name::<String>("name");
//! # }
//! ```
//!
//! Further more, if we want to iterate all workers, skipping all the other nodes, the grouping mechanism in TTGraph can come to use.
//!
//! Here, the two variant `Human` and `Robot` is in the `worker` group. Use the [`iter_group`](Graph::iter_group) method to iterate all nodes within the group.
//!
//! Notes:
//!
//! + Variants can be inside of multiple or none groups.
//! + Currently, this method does not provide performance enhancement, as it is only a wrapper on matching the variants according to the group name.
//!
//! ```rust
//! # use ttgraph::*;
//! # #[derive(TypedNode)]
//! # struct HumanWorkerNode{
//! #   name: String,
//! #   // ... other data
//! # }
//! # #[derive(TypedNode)]
//! # struct RobotWorkerNode{
//! #   name: String,
//! #   // ... other data
//! # }
//! node_enum!{
//!   enum Node{
//!     Human(HumanWorkerNode),
//!     Robot(RobotWorkerNode),
//!     // ... other nodes
//!   }
//!   group!{
//!     worker{Human, Robot},
//!   }
//! }
//!
//! # fn main() {
//! # let ctx = Context::new();
//! # let graph = Graph::<Node>::new(&ctx);
//! for (idx, node) in graph.iter_group("worker") {
//!   let name = node.data_ref_by_name::<String>("name").unwrap();
//!   // ...
//! }
//! # }
//! ```
//!
//! Links may be grouped too. Assume workers may produce different kinds of products, and make them into a `product` group can help iterate through all of them.
//!
//! Notes:
//! + A link field can be inside multiple or none groups. Syntax: `#[group(group1, group2, ...)]`
//! + Yes, its form is inconsitent with [`node_enum!`]. The problem is if a struct is inside a macro, the linter (rust-analyzer) fails to show its content. The author personally thinks the `group!` form is clearer, but does not worth ruining the linter.
//!
//! ```rust
//! # use ttgraph::*;
//! # use std::collections::BTreeSet;
//! #[derive(TypedNode)]
//! struct HumanWorkerNode{
//!   name: String,
//!   #[group(product)]
//!   cooked: BTreeSet<NodeIndex>,
//!   #[group(product)]
//!   maked: BTreeSet<NodeIndex>,
//!   // ... other data
//! }
//! #[derive(TypedNode)]
//! struct RobotWorkerNode{
//!   name: String,
//!   #[group(product)]
//!   manufactured: BTreeSet<NodeIndex>,
//!   // ... other data
//! }
//! # node_enum!{
//! #   enum Node{
//! #     Human(HumanWorkerNode),
//! #     Robot(RobotWorkerNode),
//! #     // ... other nodes
//! #   }
//! # }
//! # fn main() {
//! # let ctx = Context::new();
//! # let mut graph = Graph::<Node>::new(&ctx);
//! # let mut trans = Transaction::new(&ctx);
//! # let idx = trans.insert(Node::Robot(RobotWorkerNode{name: "robot".to_string(), manufactured: BTreeSet::new()}));
//! # graph.commit(trans);
//!
//! let node = graph.get(idx).unwrap();
//! for idx in node.get_links_by_group("product") {
//!   // Now idx binds to all NodeIndex inside the product group
//! }
//! # }
//! ```
//!
//! Other methods for type erasure are listed in the document of `NodeEnum` and `TypedNode` traits.
//!
//! ## Link type check
//!
//! A node links to other node with a [`NodeIndex`] in TTGraph, which is in fact weak typed as any variant in the node enum can be pointed by the NodeIndex.
//!
//! For debug reason, an optional link type check can be added with `link_type!{ #var.#field : #var, ... }`. When a transaction is committed, all changes which be checked. Panics if a NodeIndex points to the wrong enum variant.
//!
//! Feature `debug` is required. Otherwise all checks are skipped.
//!
//! ```rust
//! use ttgraph::*;
//! use std::collections::HashSet;
//! #[derive(TypedNode)]
//! struct FactoryNode{
//!  name: String,
//!  workers: HashSet<NodeIndex>,
//! }
//! #[derive(TypedNode)]
//! struct HumanWorkerNode{
//!   name: String,
//!   factory: NodeIndex,
//! }
//! #[derive(TypedNode)]
//! struct RobotWorkerNode{
//!   name: String,
//!   factory: NodeIndex,
//! }
//! node_enum!{
//!   enum Node{
//!     Factory(FactoryNode),
//!     Human(HumanWorkerNode),
//!     Robot(RobotWorkerNode),
//!   }
//!   link_type!{
//!     Factory.workers : {Human, Robot},
//!     Human.factory: Factory,
//!     Robot.factory: Factory,
//!   }
//! }
//! # fn main() {}
//! ```
//!
//! In this example, workers of a factory can link to human or robot, while the factory field of human and robot must link to a factory.
//!
//! ## Use group in `link_type!` and `bidirectional!`
//!
//! Groups can be used in `link_type!` and `bidirectional!`. To avoid confliction, group name should not be variant name in NodeEnum or link name in TypedNode.
//!
//! All `VarGroup.LinkGroup` will be expaneded into multiple `Var.Link` pairs of the group.
//!
//! The purpose of this feature is to greatly reduce the number of lines to describe link types and bidirectional links, especially in complex graph.
//!
//! If there are n types of the same type group and m links of the same link group, then one line of such description can replace n*m lines of trival description. (In bidirecitonal link description such line number is further squared)
//!
//! Here is a sophisticated example for to explain what this feature does.
//!
//! ```rust
//! # use ttgraph::*;
//! #[derive(TypedNode, Debug)]
//! struct Left1 {
//!   #[group(l1)]
//!   g1: NodeIndex,
//!   #[group(l2)]
//!   g2: NodeIndex,
//! }
//!
//! #[derive(TypedNode, Debug)]
//! struct Left2 {
//!   #[group(l1)]
//!   g1: NodeIndex,
//!   #[group(l2)]
//!   g2: NodeIndex,
//! }
//!
//! #[derive(TypedNode, Debug)]
//! struct Left3 {
//!   #[group(l1)]
//!   g1: NodeIndex,
//!   #[group(l2)]
//!   g2: NodeIndex,
//! }
//!
//! #[derive(TypedNode, Debug)]
//! struct Left4 {
//!   #[group(l1)]
//!   g1: NodeIndex,
//!   #[group(l2)]
//!   g2: NodeIndex,
//! }
//!
//! #[derive(TypedNode, Debug)]
//! struct Right1 {
//!   #[group(r1)]
//!   g1: NodeIndex,
//!   #[group(r2)]
//!   g2: NodeIndex,
//! }
//!
//! #[derive(TypedNode, Debug)]
//! struct Right2 {
//!   #[group(r1)]
//!   g1: NodeIndex,
//!   #[group(r2)]
//!   g2: NodeIndex,
//! }
//!
//! #[derive(TypedNode, Debug)]
//! struct Right3 {
//!   #[group(r1)]
//!   g1: NodeIndex,
//!   #[group(r2)]
//!   g2: NodeIndex,
//! }
//!
//! #[derive(TypedNode, Debug)]
//! struct Right4 {
//!   #[group(r1)]
//!   g1: NodeIndex,
//!   #[group(r2)]
//!   g2: NodeIndex,
//! }
//!
//! node_enum!{
//!   enum LR{
//!     Left1(Left1),
//!     Left2(Left2),
//!     Left3(Left3),
//!     Left4(Left4),
//!     Right1(Right1),
//!     Right2(Right2),
//!     Right3(Right3),
//!     Right4(Right4),
//!   }
//!   group!{
//!     left { Left1, Left2, Left3, Left4},
//!     right { Right1, Right2, Right3, Right4},
//!     LU {Left1, Left2},
//!     LD {Left3, Left4},
//!     RU {Right1, Right2},
//!     RD {Right3, Right4},
//!   }
//!   link_type!{
//!     left.l1: RU,
//!     left.l2: RD,
//!     right.r1: LU,
//!     right.r2: LD,
//!   }
//!   bidirectional!{
//!     LU.l1 <-> RU.r1,
//!     LD.l1 <-> RU.r2,
//!     LU.l2 <-> RD.r1,
//!     LD.l2 <-> RD.r2,
//!   }
//! }
//!
//! # fn main() {
//! let ctx = Context::new();
//! let mut graph = Graph::<LR>::new(&ctx);
//! let mut trans = Transaction::new(&ctx);
//!
//! let l1 = trans.alloc();
//! let l2 = trans.alloc();
//! let l3 = trans.alloc();
//! let l4 = trans.alloc();
//! let r1 = trans.alloc();
//! let r2 = trans.alloc();
//! let r3 = trans.alloc();
//! let r4 = trans.alloc();
//!
//! trans.fill_back(l1, LR::Left1(Left1 { g1: r1, g2: r3 }));
//! trans.fill_back(l2, LR::Left2(Left2 { g1: r2, g2: r4 }));
//! trans.fill_back(l3, LR::Left3(Left3 { g1: r1, g2: r4 }));
//! trans.fill_back(l4, LR::Left4(Left4 { g1: r2, g2: r3 }));
//!
//! trans.fill_back(r1, LR::Right1(Right1 { g1: NodeIndex::empty(), g2: NodeIndex::empty() }));
//! trans.fill_back(r2, LR::Right2(Right2 { g1: NodeIndex::empty(), g2: NodeIndex::empty() }));
//! trans.fill_back(r3, LR::Right3(Right3 { g1: NodeIndex::empty(), g2: NodeIndex::empty() }));
//! trans.fill_back(r4, LR::Right4(Right4 { g1: NodeIndex::empty(), g2: NodeIndex::empty() }));
//!
//! graph.commit(trans);
//!
//! let node = get_node!(graph, LR::Left1, l1).unwrap();
//! assert_eq!(node.g1, r1);
//! assert_eq!(node.g2, r3);
//! let node = get_node!(graph, LR::Left2, l2).unwrap();
//! assert_eq!(node.g1, r2);
//! assert_eq!(node.g2, r4);
//! let node = get_node!(graph, LR::Left3, l3).unwrap();
//! assert_eq!(node.g1, r1);
//! assert_eq!(node.g2, r4);
//! let node = get_node!(graph, LR::Left4, l4).unwrap();
//! assert_eq!(node.g1, r2);
//! assert_eq!(node.g2, r3);
//!
//! let node = get_node!(graph, LR::Right1, r1).unwrap();
//! assert_eq!(node.g1, l1);
//! assert_eq!(node.g2, l3);
//! let node = get_node!(graph, LR::Right2, r2).unwrap();
//! assert_eq!(node.g1, l2);
//! assert_eq!(node.g2, l4);
//! let node = get_node!(graph, LR::Right3, r3).unwrap();
//! assert_eq!(node.g1, l1);
//! assert_eq!(node.g2, l4);
//! let node = get_node!(graph, LR::Right4, r4).unwrap();
//! assert_eq!(node.g1, l2);
//! assert_eq!(node.g2, l3);
//! # }
//! ```
//!
//! In some cases, most types in a type-group all have a link group, but one type in that type-group does not have that link-group, which would cause compile error.
//!
//! A `phantom_group` can be used in this case to create a link group that only exists from the view of a NodeEnum.
//!
//! ```
//! # use ttgraph::*;
//! #[derive(TypedNode)]
//! #[phantom_group(b)]
//! struct A{
//!   #[group(a)]
//!   x: NodeIndex,
//! }
//!
//! #[derive(TypedNode)]
//! struct B{
//!   #[group(a)]
//!   x: NodeIndex,
//!   #[group(b)]
//!   y: NodeIndex,
//! }
//!
//! node_enum!{
//!   enum NodeAB{
//!     A(A),
//!     B(B),
//!   }
//!   group!{ All{A, B} }
//!   link_type!{
//!     All.a : A,
//!     All.b : B,
//!   }
//! }
//! # fn main() {}
//! ```
//!
//! ## Working In Progress
//!
//! + Graph creation macro. A sub-language to simplify great amount of `alloc_node`, `fill_back_node` and `new_node` calls.
//! + Graph transition. A way to conviently transit `Graph<NodeEnumA>` to `Graph<NodeEnumB>`, if `NodeEnumA` and `NodeEnumB` have a lot of common variants.
//! + Check when commit. A way to add runtime check when commit.

pub mod arena;
// pub mod graph;
mod typed_graph;

pub use typed_graph::*;
