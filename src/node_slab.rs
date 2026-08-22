use log::debug;

use crate::{
    node_index::{NodeIndex, NodeIndexNonMax},
    Node,
};

pub(crate) struct NodeSlab<NI: NodeIndex> {
    /// Maintains the mapping from [`NodeIndex`] to [`Node`]
    nodes: Vec<Node<NI>>,
    /// A stack of available node indexes that are currently not allocated to any nodes
    free_nodes: Vec<NI::NonMax>,
    /// An index within `free_nodes` pointing to the top of the stack.
    free_offset: u32,
}

impl<NI: NodeIndex> NodeSlab<NI> {
    /// Construct a new, empty `NodeSlab`
    #[inline]
    pub fn new(max_nodes: u32) -> Self {
        NodeSlab {
            nodes: vec![Node::default(); max_nodes as usize],
            // Freelist is a stack. Nodes in inverse order so that [0] pops first.
            free_nodes: (0..max_nodes)
                .map(|i| NI::NonMax::try_from(NI::from_u32(max_nodes - i - 1)).unwrap_or_default())
                .collect(),
            free_offset: max_nodes - 1,
        }
    }

    /// Return whether there is no more room for more nodes
    #[inline]
    pub fn is_full(&self) -> bool {
        self.free_offset == 0
    }

    /// Insert a node into the slab, returning the index associated with it
    #[inline]
    pub fn insert(&mut self, node: Node<NI>) -> NI::NonMax {
        assert!(!self.is_full());
        let free_offset = self.free_offset;
        let node_index = self.free_nodes[free_offset as usize];
        self.free_offset -= 1;
        debug!(
            "Getting node {} from freelist[{}]",
            node_index,
            self.free_offset + 1
        );
        self.nodes[node_index.to_usize()] = node;
        node_index
    }

    /// Remove the node associated with the index
    #[inline]
    pub fn remove(&mut self, index: NI::NonMax) {
        // Insert the removed node to freelist
        debug!(
            "Putting node {} into freelist[{}] (free)",
            index,
            self.free_offset + 1
        );
        self.free_offset += 1;
        self.free_nodes[self.free_offset as usize] = index;
    }
}

impl<NI: NodeIndex> std::ops::Index<NI::NonMax> for NodeSlab<NI> {
    type Output = Node<NI>;

    #[inline]
    fn index(&self, index: NI::NonMax) -> &Self::Output {
        &self.nodes[index.to_usize()]
    }
}

impl<NI: NodeIndex> std::ops::IndexMut<NI::NonMax> for NodeSlab<NI> {
    #[inline]
    fn index_mut(&mut self, index: NI::NonMax) -> &mut Self::Output {
        &mut self.nodes[index.to_usize()]
    }
}
