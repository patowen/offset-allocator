// offset-allocator/src/lib.rs

#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::{Debug, Display, Formatter};

use log::debug;

use crate::small_float::{SmallFloat, SmallFloatMap};

const NUM_TOP_BINS: usize = 32;
const TOP_BINS_INDEX_SHIFT: u32 = 3;
const LEAF_BINS_INDEX_MASK: u32 = 7;

/// Determines the number of allocations that the allocator supports.
///
/// By default, [`Allocator`] and related functions use `u32`, which allows for
/// `u32::MAX - 1` allocations. You can, however, use `u16` instead, which
/// causes the allocator to use less memory but limits the number of allocations
/// within a single allocator to at most 65,534.
pub trait NodeIndex: Display + Debug + Clone + Copy + PartialEq + Eq {
    /// An invalid representation in its type, used as the `None` type of `NodeIndexOption`.
    const INVALID: Self;

    /// The number of indexes, consectuive starting from 0, that are valid representations
    const NUM_VALID: u32;

    /// Converts from a unsigned 32-bit integer to an instance of this type.
    fn from_u32(val: u32) -> Self;

    /// Converts this type to an unsigned machine word.
    fn to_usize(self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeIndexOption<NI: NodeIndex>(NI);

impl<NI: NodeIndex> NodeIndexOption<NI> {
    const NONE: Self = NodeIndexOption(NodeIndex::INVALID);

    fn some(inner: NI) -> Self {
        Self(inner)
    }

    #[inline]
    fn to_option(self) -> Option<NI> {
        if self == Self::NONE {
            None
        } else {
            Some(self.0)
        }
    }

    #[inline]
    fn is_none(self) -> bool {
        self == Self::NONE
    }

    #[inline]
    fn unwrap(self) -> NI {
        assert!(self != Self::NONE);
        self.0
    }
}

impl<NI: NodeIndex> Default for NodeIndexOption<NI> {
    fn default() -> Self {
        Self::NONE
    }
}

/// An allocator that manages a single contiguous chunk of space and hands out
/// portions of it as requested.
pub struct Allocator<NI: NodeIndex = u32> {
    size: u32,
    max_allocs: u32,
    /// How much available space there is across all nodes
    free_storage: u32,

    bins_map: BinsMap<NI>,

    nodes: NodeMap<NI>,
    free_nodes: FreeNodeStack<NI>,
}

/// A map from each bin to the node at the head of the linked list for that bin. The name of this struct is `BinsMap` instead of `BinMap` to avoid confusion with binary maps.
struct BinsMap<NI: NodeIndex> {
    /// (Patrick) A bit-vector showing which `top_bin_index`es are "used"
    occupied_bins_top: u32,
    /// (Patrick) An array of 32 bit-vectors showing which `leaf_bin_index`es are "used" for the given top bin, usually indexed by `top_bin_index`
    occupied_bins: [u8; NUM_TOP_BINS],
    /// (Patrick) An array of 256 `node_index`es (each being a head of a doubly-linked list of nodes in that bin), usually indexed by `bin_index` (a combo of `top_bin_index` and `leaf_bin_index`).
    head_nodes: SmallFloatMap<NodeIndexOption<NI>>,
}

impl<NI: NodeIndex> Default for BinsMap<NI> {
    fn default() -> Self {
        Self {
            occupied_bins_top: 0,
            occupied_bins: [0; NUM_TOP_BINS],
            head_nodes: SmallFloatMap::default(),
        }
    }
}

impl<NI: NodeIndex> BinsMap<NI> {
    fn min_occupied_since(&self, min: SmallFloat) -> Option<SmallFloat> {
        let min_top_bin_index = min.reinterpret_as_u32() >> TOP_BINS_INDEX_SHIFT;
        let min_leaf_bin_index = min.reinterpret_as_u32() & LEAF_BINS_INDEX_MASK;

        let mut top_bin_index = min_top_bin_index;
        let mut leaf_bin_index = None;

        // If top bin exists, scan its leaf bin. This can fail (NO_SPACE).
        if (self.occupied_bins_top & (1 << top_bin_index)) != 0 {
            leaf_bin_index = find_lowest_bit_set_after(
                self.occupied_bins[top_bin_index as usize] as _,
                min_leaf_bin_index,
            );
        }

        // If we didn't find space in top bin, we search top bin from +1
        let leaf_bin_index = match leaf_bin_index {
            Some(leaf_bin_index) => leaf_bin_index,
            None => {
                top_bin_index =
                    find_lowest_bit_set_after(self.occupied_bins_top, min_top_bin_index + 1)?;

                // All leaf bins here fit the alloc, since the top bin was
                // rounded up. Start leaf search from bit 0.
                //
                // NOTE: This search can't fail since at least one leaf bit was
                // set because the top bit was set.
                self.occupied_bins[top_bin_index as usize].trailing_zeros()
            }
        };

        Some(SmallFloat::reinterpret_u32(
            (top_bin_index << TOP_BINS_INDEX_SHIFT) | leaf_bin_index,
        ))
    }

    fn max_occupied(&self) -> Option<SmallFloat> {
        if self.occupied_bins_top == 0 {
            return None;
        }
        let top_bin_index = self.occupied_bins_top.ilog2();
        let leaf_bin_index = (self.occupied_bins[top_bin_index as usize] as u32).ilog2();
        Some(SmallFloat::reinterpret_u32(
            (top_bin_index << TOP_BINS_INDEX_SHIFT) | leaf_bin_index,
        ))
    }

    fn mark_bin_empty(&mut self, bin_index: SmallFloat) {
        let top_bin_index = bin_index.reinterpret_as_u32() >> TOP_BINS_INDEX_SHIFT;
        let leaf_bin_index = bin_index.reinterpret_as_u32() & LEAF_BINS_INDEX_MASK;

        // Remove a leaf bin mask bit
        self.occupied_bins[top_bin_index as usize] &= !(1 << u32::from(leaf_bin_index));

        // All leaf bins empty?
        if self.occupied_bins[top_bin_index as usize] == 0 {
            // Remove a top bin mask bit
            self.occupied_bins_top &= !(1 << top_bin_index);
        }
    }

    fn mark_bin_occupied(&mut self, bin_index: SmallFloat) {
        let top_bin_index = bin_index.reinterpret_as_u32() >> TOP_BINS_INDEX_SHIFT;
        let leaf_bin_index = bin_index.reinterpret_as_u32() & LEAF_BINS_INDEX_MASK;

        // Set bin mask bits
        self.occupied_bins[top_bin_index as usize] |= 1 << leaf_bin_index;
        self.occupied_bins_top |= 1 << top_bin_index;
    }
}

impl<NI: NodeIndex> std::ops::Index<SmallFloat> for BinsMap<NI> {
    type Output = NodeIndexOption<NI>;

    fn index(&self, index: SmallFloat) -> &Self::Output {
        &self.head_nodes[index]
    }
}

impl<NI: NodeIndex> std::ops::IndexMut<SmallFloat> for BinsMap<NI> {
    fn index_mut(&mut self, index: SmallFloat) -> &mut Self::Output {
        &mut self.head_nodes[index]
    }
}

struct NodeMap<NI: NodeIndex>(Vec<Node<NI>>);

impl<NI: NodeIndex> NodeMap<NI> {
    fn with_max_allocs(max_allocs: u32) -> Self {
        Self(vec![Node::default(); max_allocs as usize])
    }
}

impl<NI: NodeIndex> std::ops::Index<NI> for NodeMap<NI> {
    type Output = Node<NI>;

    fn index(&self, index: NI) -> &Self::Output {
        &self.0[index.to_usize()]
    }
}

impl<NI: NodeIndex> std::ops::IndexMut<NI> for NodeMap<NI> {
    fn index_mut(&mut self, index: NI) -> &mut Self::Output {
        &mut self.0[index.to_usize()]
    }
}

struct FreeNodeStack<NI: NodeIndex>(Vec<NI>);

impl<NI: NodeIndex> FreeNodeStack<NI> {
    fn with_max_allocs(max_allocs: u32) -> Self {
        Self((0..max_allocs).rev().map(|i| NI::from_u32(i)).collect())
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    fn push(&mut self, node_index: NI) {
        debug!(
            "Putting node {} into freelist[{}] (free)",
            node_index,
            self.0.len()
        );
        self.0.push(node_index);
    }

    #[inline]
    fn pop_required(&mut self) -> NI {
        let node_index = self.0.pop().expect("Stack must not be exhausted");
        debug!(
            "Getting node {} from freelist[{}]",
            node_index,
            self.0.len()
        );
        node_index
    }
}

/// A single allocation.
#[derive(Clone, Copy)]
pub struct Allocation<NI = u32>
where
    NI: NodeIndex,
{
    /// The location of this allocation within the buffer.
    pub offset: u32,
    /// The node index associated with this allocation.
    metadata: NI,
}

/// Provides a summary of the state of the allocator, including space remaining.
#[derive(Debug)]
pub struct StorageReport {
    /// The amount of free space left.
    pub total_free_space: u32,
    /// The maximum potential size of a single contiguous allocation.
    pub largest_free_region: u32,
}

/// Provides a detailed accounting of each bin within the allocator.
#[derive(Debug)]
pub struct StorageReportFull {
    /// Each bin within the allocator.
    pub free_regions: SmallFloatMap<StorageReportFullRegion>,
}

/// A detailed accounting of each allocator bin.
#[derive(Clone, Copy, Debug, Default)]
pub struct StorageReportFullRegion {
    /// The size of the bin, in units.
    pub size: u32,
    /// The number of allocations in the bin.
    pub count: u32,
}

#[derive(Clone, Copy)]
struct Node<NI = u32>
where
    NI: NodeIndex,
{
    /// The offset of the node in the address space of the heap
    data_offset: u32,
    /// The size of the node in the address space of the heap
    data_size: u32,
    /// (Patrick) Part of a linked list. Stores the previous `node_index` in the same bin
    bin_list_prev: NodeIndexOption<NI>,
    /// (Patrick) Part of a linked list. Stores the next `node_index` in the same bin
    bin_list_next: NodeIndexOption<NI>,
    /// (Patrick) Part of a linked list. Stores the previous `node_index` contiguously in memory
    neighbor_prev: NodeIndexOption<NI>,
    /// (Patrick) Part of a linked list. Stores the next `node_index` contiguously in memory
    neighbor_next: NodeIndexOption<NI>,
    /// (Patrick) Whether the node is used in an active allocation (rather than being a free slot). If `true`, this node is no longer in a bin.
    used: bool, // TODO: Merge as bit flag
}

impl<NI: NodeIndex> Default for Node<NI> {
    fn default() -> Self {
        Self {
            data_offset: Default::default(),
            data_size: Default::default(),
            bin_list_prev: Default::default(),
            bin_list_next: Default::default(),
            neighbor_prev: Default::default(),
            neighbor_next: Default::default(),
            used: Default::default(),
        }
    }
}

// Utility functions
/// Find the lowest bit that is set to 1, as long as it's at least start_bit_index. Return `None` if there is no such bit.
fn find_lowest_bit_set_after(bit_mask: u32, start_bit_index: u32) -> Option<u32> {
    let mask_before_start_index = (1 << start_bit_index) - 1;
    let mask_after_start_index = !mask_before_start_index;
    let bits_after = bit_mask & mask_after_start_index;
    if bits_after == 0 {
        None
    } else {
        Some(bits_after.trailing_zeros())
    }
}

impl<NI> Allocator<NI>
where
    NI: NodeIndex,
{
    /// Creates a new allocator, managing a contiguous block of memory of `size`
    /// units, with a default reasonable number of maximum allocations.
    pub fn new(size: u32) -> Self {
        Allocator::with_max_allocs(size, u32::min(128 * 1024, NI::NUM_VALID))
    }

    /// Creates a new allocator, managing a contiguous block of memory of `size`
    /// units, with the given number of maximum allocations.
    ///
    /// Note that the maximum number of allocations must be less than
    /// [`NodeIndex::MAX`] minus one. If this restriction is violated, this
    /// constructor will panic.
    pub fn with_max_allocs(size: u32, max_allocs: u32) -> Self {
        assert!(max_allocs < NI::NUM_VALID);

        let mut this = Self {
            size,
            max_allocs,
            free_storage: 0,
            bins_map: BinsMap::default(),
            nodes: NodeMap::with_max_allocs(max_allocs),
            free_nodes: FreeNodeStack::with_max_allocs(max_allocs),
        };
        this.insert_node_into_bin(size, 0);
        this
    }

    /// Clears out all allocations.
    pub fn reset(&mut self) {
        *self = Self::with_max_allocs(self.size, self.max_allocs);
    }

    /// Allocates a block of `size` elements and returns its allocation.
    ///
    /// If there's not enough contiguous space for this allocation, returns
    /// None.
    pub fn allocate(&mut self, size: u32) -> Option<Allocation<NI>> {
        // Out of allocations?
        if self.free_nodes.is_exhausted() {
            // TODO: Do we want to allow an allocation that doesn't create a new node?
            return None;
        }

        // Round up to bin index to ensure that alloc >= bin
        // Gives us min bin index that fits the size
        let min_bin_index = SmallFloat::from_u32_round_up(size);
        let bin_index = self.bins_map.min_occupied_since(min_bin_index)?;

        // Pop the top node of the bin. Bin top = node.next.
        let node_index = self.bins_map[bin_index].unwrap();
        let node = &mut self.nodes[node_index];
        let node_total_size = node.data_size;
        node.data_size = size;
        node.used = true;
        self.bins_map[bin_index] = node.bin_list_next; // TODO: Doesn't this need IndexMut?
        if let Some(bin_list_next) = node.bin_list_next.to_option() {
            self.nodes[bin_list_next].bin_list_prev = NodeIndexOption::NONE;
        }
        self.free_storage -= node_total_size;
        debug!(
            "Free storage: {} (-{}) (allocate)",
            self.free_storage, node_total_size
        );

        // Bin empty?
        if self.bins_map[bin_index].is_none() {
            self.bins_map.mark_bin_empty(bin_index);
        }

        // Push back remainder N elements to a lower bin
        let remainder_size = node_total_size - size;
        if remainder_size > 0 {
            let Node {
                data_offset,
                neighbor_next,
                ..
            } = self.nodes[node_index];

            let new_node_index = self.insert_node_into_bin(remainder_size, data_offset + size);

            // Link nodes next to each other so that we can merge them later if both are free
            // And update the old next neighbor to point to the new node (in middle)
            let node = &mut self.nodes[node_index];
            if let Some(neighbor_next) = node.neighbor_next.to_option() {
                self.nodes[neighbor_next].neighbor_prev = NodeIndexOption::some(new_node_index);
            }
            self.nodes[new_node_index].neighbor_prev = NodeIndexOption::some(node_index);
            self.nodes[new_node_index].neighbor_next = neighbor_next;
            self.nodes[node_index].neighbor_next = NodeIndexOption::some(new_node_index);
        }

        let node = &mut self.nodes[node_index];
        Some(Allocation {
            offset: node.data_offset,
            metadata: node_index,
        })
    }

    /// Frees an allocation, returning the data to the heap.
    ///
    /// If the allocation has already been freed, the behavior is unspecified.
    /// It may or may not panic. Note that, because this crate contains no
    /// unsafe code, the memory safe of the allocator *itself* will be
    /// uncompromised, even on double free.
    pub fn free(&mut self, allocation: Allocation<NI>) {
        let node_index = allocation.metadata;

        // Merge with neighbors…
        let Node {
            data_offset: mut offset,
            data_size: mut size,
            used,
            ..
        } = self.nodes[node_index];

        // Double delete check
        assert!(used);

        if let Some(neighbor_prev) = self.nodes[node_index].neighbor_prev.to_option() {
            if !self.nodes[neighbor_prev].used {
                // Previous (contiguous) free node: Change offset to previous
                // node offset. Sum sizes
                let prev_node = &self.nodes[neighbor_prev];
                offset = prev_node.data_offset;
                size += prev_node.data_size;

                // Remove node from the bin linked list and put it in the
                // freelist
                self.remove_node_from_bin(neighbor_prev);

                let prev_node = &self.nodes[neighbor_prev];
                debug_assert_eq!(prev_node.neighbor_next, NodeIndexOption::some(node_index));
                self.nodes[node_index].neighbor_prev = prev_node.neighbor_prev;
            }
        }

        if let Some(neighbor_next) = self.nodes[node_index].neighbor_next.to_option() {
            if !self.nodes[neighbor_next].used {
                // Next (contiguous) free node: Offset remains the same. Sum
                // sizes.
                let next_node = &self.nodes[neighbor_next];
                size += next_node.data_size;

                // Remove node from the bin linked list and put it in the
                // freelist
                self.remove_node_from_bin(neighbor_next);

                let next_node = &self.nodes[neighbor_next];
                debug_assert_eq!(next_node.neighbor_prev, NodeIndexOption::some(node_index));
                self.nodes[node_index].neighbor_next = next_node.neighbor_next;
            }
        }

        let Node {
            neighbor_next,
            neighbor_prev,
            ..
        } = self.nodes[node_index];

        // Insert the removed node to freelist
        self.free_nodes.push(node_index);

        // Insert the (combined) free node to bin
        let combined_node_index = self.insert_node_into_bin(size, offset);

        // Connect neighbors with the new combined node
        if let Some(neighbor_next) = neighbor_next.to_option() {
            self.nodes[combined_node_index].neighbor_next = NodeIndexOption::some(neighbor_next);
            self.nodes[neighbor_next].neighbor_prev = NodeIndexOption::some(combined_node_index);
        }
        if let Some(neighbor_prev) = neighbor_prev.to_option() {
            self.nodes[combined_node_index].neighbor_prev = NodeIndexOption::some(neighbor_prev);
            self.nodes[neighbor_prev].neighbor_next = NodeIndexOption::some(combined_node_index);
        }
    }

    fn insert_node_into_bin(&mut self, size: u32, data_offset: u32) -> NI {
        // Round down to bin index to ensure that bin >= alloc
        let bin_index = SmallFloat::from_u32_round_down(size);

        // Bin was empty before?
        if self.bins_map[bin_index].is_none() {
            // Set bin mask bits
            self.bins_map.mark_bin_occupied(bin_index);
        }

        // Take a freelist node and insert on top of the bin linked list (next = old top)
        let top_node_index = self.bins_map[bin_index];
        let node_index = self.free_nodes.pop_required();
        self.nodes[node_index] = Node {
            data_offset,
            data_size: size,
            bin_list_next: top_node_index,
            ..Node::default()
        };
        if let Some(top_node_index) = top_node_index.to_option() {
            self.nodes[top_node_index].bin_list_prev = NodeIndexOption::some(node_index);
        }
        self.bins_map[bin_index] = NodeIndexOption::some(node_index);

        self.free_storage += size;
        debug!(
            "Free storage: {} (+{}) (insert_node_into_bin)",
            self.free_storage, size
        );
        node_index
    }

    fn remove_node_from_bin(&mut self, node_index: NI) {
        // Copy the node to work around borrow check.
        let node = self.nodes[node_index];

        match node.bin_list_prev.to_option() {
            Some(bin_list_prev) => {
                // Easy case: We have previous node. Just remove this node from the middle of the list.
                self.nodes[bin_list_prev].bin_list_next = node.bin_list_next;
                if let Some(bin_list_next) = node.bin_list_next.to_option() {
                    self.nodes[bin_list_next].bin_list_prev = node.bin_list_prev;
                }
            }
            None => {
                // Hard case: We are the first node in a bin. Find the bin.

                // Round down to bin index to ensure that bin >= alloc
                let bin_index = SmallFloat::from_u32_round_down(node.data_size);

                self.bins_map[bin_index] = node.bin_list_next;
                if let Some(bin_list_next) = node.bin_list_next.to_option() {
                    self.nodes[bin_list_next].bin_list_prev = NodeIndexOption::NONE;
                }

                // Bin empty?
                if self.bins_map[bin_index].is_none() {
                    self.bins_map.mark_bin_empty(bin_index);
                }
            }
        }

        // Insert the node to freelist
        self.free_nodes.push(node_index);

        self.free_storage -= node.data_size;
        debug!(
            "Free storage: {} (-{}) (remove_node_from_bin)",
            self.free_storage, node.data_size
        );
    }

    /// Returns the *used* size of an allocation.
    ///
    /// Note that this may be larger than the size requested at allocation time,
    /// due to rounding. (Patrick) No, it's never larger.
    pub fn allocation_size(&self, allocation: Allocation<NI>) -> u32 {
        self.nodes[allocation.metadata].data_size
    }

    /// Returns a structure containing the amount of free space remaining, as
    /// well as the largest amount that can be allocated at once.
    pub fn storage_report(&self) -> StorageReport {
        if self.free_nodes.is_exhausted() {
            // Out of allocations? -> Zero free space
            return StorageReport {
                total_free_space: 0,
                largest_free_region: 0,
            };
        }

        let largest_free_region = self.bins_map.max_occupied().map_or(0, |x| x.to_u32());
        debug_assert!(self.free_storage >= largest_free_region);

        StorageReport {
            total_free_space: self.free_storage,
            largest_free_region,
        }
    }

    /// Returns detailed information about the number of allocations in each
    /// bin.
    pub fn storage_report_full(&self) -> StorageReportFull {
        let mut report = StorageReportFull::default();
        for i in SmallFloat::values() {
            let mut count = 0;
            let mut maybe_node_index = self.bins_map[i];
            while let Some(node_index) = maybe_node_index.to_option() {
                maybe_node_index = self.nodes[node_index].bin_list_next;
                count += 1;
            }
            report.free_regions[i] = StorageReportFullRegion {
                size: i.to_u32(),
                count,
            }
        }
        report
    }
}

impl Default for StorageReportFull {
    fn default() -> Self {
        Self {
            free_regions: SmallFloatMap::default(),
        }
    }
}

impl<NI> Debug for Allocator<NI>
where
    NI: NodeIndex,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.storage_report().fmt(f)
    }
}

impl NodeIndex for u32 {
    const INVALID: u32 = u32::MAX;

    const NUM_VALID: u32 = Self::INVALID;

    fn from_u32(val: u32) -> Self {
        assert!(val < Self::NUM_VALID);
        val
    }

    fn to_usize(self) -> usize {
        self as usize
    }
}

impl NodeIndex for u16 {
    const INVALID: u16 = u16::MAX;

    const NUM_VALID: u32 = Self::INVALID as u32;

    fn from_u32(val: u32) -> Self {
        assert!(val < Self::NUM_VALID);
        val as u16
    }

    fn to_usize(self) -> usize {
        self as usize
    }
}
