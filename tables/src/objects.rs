//! The module  library that provides functionalities for managing and querying
//! a graph of real-world objects.

use std::{collections::{HashMap, HashSet, VecDeque}, ops::Deref};

use compact_str::CompactString;
use nimi_pona::NimiPonaConvert;
use petgraph::{Direction, visit::{GraphBase, IntoNeighbors, Visitable}};

use crate::strings::{Language, LanguageResolution, Localized, Name};



/// Unique ID of a real world object defined in [`ObjectGraph`]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ObjectId(pub(super) u128);


impl ObjectId {
    /// Create a new uuid
    pub fn create_new() -> Self
    {
        let id = uuid::Uuid::new_v4();
        Self(id.as_u128())
    }
}

impl NimiPonaConvert for ObjectId {
    fn display_nimipona(self) -> impl std::fmt::Display {
        self.0.display_nimipona()
    }

    fn parse_nimipona(str: &str) -> Result<Self, nimi_pona::NimiPonaParseError> {
        Ok(Self(u128::parse_nimipona(str)?))
    }
}


/// A graph node containing info about specific real world object
pub struct ObjectNode {
    /// Short name of the concept, i18n-ized
    pub name: Localized<Name>,
    /// Definition and description of the concept, i18n-ized
    pub description: Localized<CompactString>,

    /// The first-hand parents of this node
    pub direct_parents: Vec<ObjectId>,
    /// The first-hand children of this node, must be consistent with parents
    pub direct_children: Vec<ObjectId>,
}


/// A storage for concepts representing real world objects
pub struct ObjectGraph {
    object_map: HashMap<ObjectId, ObjectNode>,
}


#[allow(missing_docs)]
#[derive(Debug)]
pub enum AdditionError {
    NameExists
}

impl Default for ObjectGraph { fn default() -> Self { Self::new() } }
impl ObjectGraph {
    /// Create a new empty object graph
    pub fn new() -> Self {
        ObjectGraph { object_map: HashMap::new() }
    }

    /// Find an object by its name on some language
    pub fn find_name(&self, name: &<Name as Deref>::Target, lang: &LanguageResolution) -> Option<ObjectId> {
        self.object_map.iter().find_map(|(&key, def)| if def.name.get(lang).str == name {Some(key)} else {None})
    }

    /// Check if a node is the descendant of another
    pub fn is_child_of(&self, id: ObjectId, prop_parent: ObjectId) -> bool {
        petgraph::algo::has_path_connecting(self, prop_parent, id, None)
    }

    /// Create a new object node not linked to others
    pub fn define_new_object(&mut self, lang: Language, name: Name, description: CompactString) -> Result<ObjectId, AdditionError> {
        if self.object_map.iter().any(|(_id, ex)| ex.name.get(&LanguageResolution { primary: lang, secondary: None }).str == name.deref()) {
            return Err(AdditionError::NameExists);
        }

        let def = ObjectNode {
            name: Localized::new(lang, name),
            description: Localized::new(lang, description),
            direct_children: vec![],
            direct_parents: vec![],
        };

        let id = ObjectId::create_new();

        self.object_map.insert(id, def);

        Ok(id)
    }

    /// Relink hierarchy so these two objects are parent and child
    pub fn claim_child(&mut self, id: ObjectId, prop_parent: ObjectId) -> Result<(), String> {
        if id == prop_parent {
            return Err("Cannot attach as child to itself".to_string());
        }

        if self.is_child_of(prop_parent, id) {
            return Err("Cannot attach as child to own descendant".to_string())
        }

        if let Some(id_data) = self.object_map.get_mut(&id) {
            id_data.direct_children.retain(|&x| x != prop_parent);
            id_data.direct_parents.retain(|&x| x != prop_parent);
        }
        if let Some(parent_data) = self.object_map.get_mut(&prop_parent) {
            parent_data.direct_parents.retain(|&x| x != id);
            parent_data.direct_children.retain(|&x| x != id);
        }

        if let Some(id_data) = self.object_map.get_mut(&id) {
            id_data.direct_parents.push(prop_parent);
        }
        if let Some(parent_data) = self.object_map.get_mut(&prop_parent) {
            parent_data.direct_children.push(id);
        }

        Ok(())
    }

    /// Iterate over all objects
    pub fn iter(&self) -> impl Iterator<Item=(&ObjectId, &ObjectNode)> {
        self.object_map.iter()
    }

    /// Iterate over objects having no parent
    pub fn roots(&self) -> impl Iterator<Item=ObjectId> {
        self.object_map.iter().filter_map(|o| if o.1.direct_parents.is_empty() {Some(o.0)} else {None}).copied()
    }

    /// Find common linked objects (Incoming -> LCA, Outgoing -> HCD)
    pub fn find_common_bounds(&self, targets: &[ObjectId], dir: Direction) -> Vec<ObjectId> {
        if targets.is_empty() {
            return Vec::new();
        }

        let get_neighbors = |id: &ObjectId| -> &[ObjectId] {
            if let Some(data) = self.object_map.get(id) {
                match dir {
                    Direction::Incoming => &data.direct_parents,
                    Direction::Outgoing => &data.direct_children,
                }
            } else {
                &[]
            }
        };

        let mut common_nodes: Option<HashSet<ObjectId>> = None;

        for &target in targets {
            let mut visited_nodes = HashSet::new();
            let mut queue = VecDeque::new();

            visited_nodes.insert(target);
            queue.push_back(target);

            while let Some(current) = queue.pop_front() {
                for &next_node in get_neighbors(&current) {
                    if visited_nodes.insert(next_node) {
                        queue.push_back(next_node);
                    }
                }
            }

            if let Some(ref mut common) = common_nodes {
                common.retain(|node| visited_nodes.contains(node));
            } else {
                common_nodes = Some(visited_nodes);
            }
        }

        let common_nodes = match common_nodes {
            Some(set) if !set.is_empty() => set,
            _ => return Vec::new(),
        };

        let mut strict_bounds = HashSet::new();

        for &node in &common_nodes {
            let mut queue = VecDeque::new();

            for &next_node in get_neighbors(&node) {
                if strict_bounds.insert(next_node) {
                    queue.push_back(next_node);
                }
            }

            while let Some(current) = queue.pop_front() {
                for &next_node in get_neighbors(&current) {
                    if strict_bounds.insert(next_node) {
                        queue.push_back(next_node);
                    }
                }
            }
        }

        common_nodes
            .into_iter()
            .filter(|node| !strict_bounds.contains(node))
            .collect()
    }

    /// Find Lowest Common Ancestor
    pub fn find_lcas(&self, targets: &[ObjectId]) -> Vec<ObjectId> {
        self.find_common_bounds(targets, Direction::Incoming)
    }

    /// Find Highest Common Descendant
    pub fn find_hcd(&self, targets: &[ObjectId]) -> Vec<ObjectId> {
        self.find_common_bounds(targets, Direction::Outgoing)
    }
}

impl<'a> IntoNeighbors for &'a ObjectGraph {
    type Neighbors = std::iter::Cloned<std::slice::Iter<'a, ObjectId>>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.object_map
            .get(&a)
            .map(|data| data.direct_children.iter().cloned())
            .unwrap_or_else(|| [].iter().cloned())
    }
}

impl petgraph::visit::IntoNeighborsDirected for &'_ ObjectGraph {
    type NeighborsDirected = Self::Neighbors;
    fn neighbors_directed(self, a: Self::NodeId, dir: petgraph::Direction) -> Self::Neighbors {
        self.object_map
            .get(&a)
            .map(|data| match dir {
                petgraph::Direction::Outgoing => data.direct_children.iter().cloned(),
                petgraph::Direction::Incoming => data.direct_parents.iter().cloned(),
            })
            .unwrap_or_else(|| [].iter().cloned())
    }
}

impl core::ops::Index<ObjectId> for ObjectGraph {
    type Output = ObjectNode;

    fn index(&self, index: ObjectId) -> &Self::Output {
        self.object_map.index(&index)
    }
}

/// Representation of an edge in an object DAG
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    source: ObjectId,
    target: ObjectId,
}


impl petgraph::prelude::EdgeRef for GraphEdge {
    type NodeId = ObjectId;
    type EdgeId = GraphEdge;
    type Weight = ();

    fn source(&self) -> Self::NodeId {
        self.source
    }

    fn target(&self) -> Self::NodeId {
        self.target
    }

    fn weight(&self) -> &Self::Weight {
        &()
    }

    fn id(&self) -> Self::EdgeId { *self }
}

struct GraphEdgesIter {
    source: ObjectId,
    direction: Direction,
    inner: std::iter::Cloned<std::slice::Iter<'static, ObjectId>>,
}

impl Iterator for &'_ mut GraphEdgesIter {
    type Item = GraphEdge;

    fn next(&mut self) -> Option<Self::Item> {
        let neighbor = self.inner.next()?;
        match self.direction {
            Direction::Outgoing => Some(GraphEdge {
                source: self.source,
                target: neighbor,
            }),
            Direction::Incoming => Some(GraphEdge {
                source: neighbor,
                target: self.source,
            }),
        }
    }
}

impl petgraph::visit::Data for &'_ ObjectGraph {
    type NodeWeight = ();
    type EdgeWeight = ();
}

impl<'a> petgraph::visit::IntoEdgeReferences for &'a ObjectGraph {
    type EdgeRef = GraphEdge;
    type EdgeReferences = Box<dyn Iterator<Item = Self::EdgeRef> + 'a>;

    fn edge_references(self) -> Self::EdgeReferences {
        let iter = self.object_map.iter().flat_map(|(&source_id, data)| {
            data.direct_children.iter().map(move |&target_id| GraphEdge {
                source: source_id,
                target: target_id,
            })
        });

        Box::new(iter)
    }
}

impl<'a> petgraph::visit::IntoEdges for &'a ObjectGraph {
    type Edges = Box<dyn Iterator<Item = Self::EdgeRef> + 'a>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        use petgraph::visit::IntoEdgesDirected;
        self.edges_directed(a, Direction::Outgoing)
    }
}

impl<'a> petgraph::visit::IntoEdgesDirected for &'a ObjectGraph {
    type EdgesDirected = Box<dyn Iterator<Item = Self::EdgeRef> + 'a>;

    fn edges_directed(self, a: Self::NodeId, dir: Direction) -> Self::EdgesDirected {
        let iter = self.object_map
            .get(&a)
            .map(|data| match dir {
                Direction::Outgoing => data.direct_children.iter().cloned(),
                Direction::Incoming => data.direct_parents.iter().cloned(),
            })
            .unwrap_or_else(|| [].iter().cloned());

        let direction = dir;
        Box::new(iter.map(move |neighbor| match direction {
            Direction::Outgoing => GraphEdge { source: a, target: neighbor },
            Direction::Incoming => GraphEdge { source: neighbor, target: a },
        }))
    }
}

impl GraphBase for ObjectGraph {
    type EdgeId = GraphEdge;
    type NodeId = ObjectId;
}


impl Visitable for ObjectGraph {
    type Map = HashSet<Self::NodeId>;

    fn visit_map(&self) -> Self::Map { HashSet::new() }
    fn reset_map(&self, map: &mut Self::Map) { map.clear(); }
}


#[test]
fn test_objects() {
    let en = Language::new("en").unwrap();
    let mut graph = ObjectGraph::new();
    let p1 = graph.define_new_object(en, Name::new("Parent1").unwrap(), "description".into()).unwrap();
    let p2 = graph.define_new_object(en, Name::new("Parent2").unwrap(), "description".into()).unwrap();
    let c1 = graph.define_new_object(en, Name::new("Child1").unwrap(), "description".into()).unwrap();
    let c2 = graph.define_new_object(en, Name::new("Child2").unwrap(), "description".into()).unwrap();

    assert!(!graph.is_child_of(c1, p1));
    assert!(!graph.is_child_of(p1, c1));
    assert!(!graph.is_child_of(c2, p1));
    assert!(!graph.is_child_of(p1, c2));
    assert!(!graph.is_child_of(c1, c2));
    assert!(!graph.is_child_of(c2, c1));

    graph.claim_child(c1, p1).unwrap();

    assert!(graph.is_child_of(c1, p1));
    assert!(!graph.is_child_of(c1, p2));
    assert!(!graph.is_child_of(c2, p1));
    assert!(!graph.is_child_of(p1, c1));

    assert!(graph.find_common_bounds(&[c1, c2], Direction::Incoming).into_iter().collect::<HashSet<_>>() == vec![].into_iter().collect::<HashSet<_>>());
    assert!(graph.find_common_bounds(&[p1, p2], Direction::Outgoing).into_iter().collect::<HashSet<_>>() == vec![].into_iter().collect::<HashSet<_>>());

    graph.claim_child(c1, p2).unwrap();

    assert!(graph.is_child_of(c1, p1));
    assert!(graph.is_child_of(c1, p2));
    assert!(!graph.is_child_of(c2, p1));
    assert!(!graph.is_child_of(p1, c1));

    assert!(graph.find_common_bounds(&[c1, c2], Direction::Incoming).into_iter().collect::<HashSet<_>>() == vec![].into_iter().collect::<HashSet<_>>());
    assert!(graph.find_common_bounds(&[p1, p2], Direction::Outgoing).into_iter().collect::<HashSet<_>>() == vec![c1].into_iter().collect::<HashSet<_>>());

    graph.claim_child(c2, p2).unwrap();

    assert!(graph.find_common_bounds(&[c1, c2], Direction::Incoming).into_iter().collect::<HashSet<_>>() == vec![p2].into_iter().collect::<HashSet<_>>());
    assert!(graph.find_common_bounds(&[p1, p2], Direction::Outgoing).into_iter().collect::<HashSet<_>>() == vec![c1].into_iter().collect::<HashSet<_>>());

    graph.claim_child(c2, p1).unwrap();

    assert!(graph.find_common_bounds(&[c1, c2], Direction::Incoming).into_iter().collect::<HashSet<_>>() == vec![p1, p2].into_iter().collect::<HashSet<_>>());
    assert!(graph.find_common_bounds(&[p1, p2], Direction::Outgoing).into_iter().collect::<HashSet<_>>() == vec![c1, c2].into_iter().collect::<HashSet<_>>());
}
