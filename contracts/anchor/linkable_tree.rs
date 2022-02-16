use super::*;
use ink_prelude::vec::Vec;
use ink_storage::collections::HashMap;
use ink_storage::traits::{SpreadLayout, PackedLayout};
#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;

pub type ChainId = u64;

#[derive(Default, Debug, SpreadLayout)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct LinkableMerkleTree {
    pub max_edges: u32,
    pub edges: HashMap<ChainId, Edge>,
    pub curr_neighbor_root_index: HashMap<ChainId, u32>,
    pub neighbor_roots: HashMap<(ChainId, u32), [u8; 32]>,
}

#[derive(Default, Debug, Clone, SpreadLayout, PackedLayout, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(StorageLayout, scale_info::TypeInfo))]
pub struct Edge {
    pub chain_id: ChainId,
    pub root: [u8; 32],
    pub latest_leaf_index: u32,
}


const ROOT_HISTORY_SIZE: u32 = 100;

impl LinkableMerkleTree {
    fn has_edge(&self, chain_id: ChainId) -> bool {
        self.edges.contains_key(&chain_id)
    }

    pub fn update_edge(
        &mut self,
        edge: Edge,
    ) -> anchor::Result<()> {
        if self.has_edge(edge.chain_id) {
            assert!(
                edge.latest_leaf_index < self.edges[&edge.chain_id].latest_leaf_index + 65_536,
                "latest leaf index should be greater than the previous one"
            );
            self.edges.insert(edge.chain_id, edge.clone());
            let neighbor_root_index = (self.curr_neighbor_root_index[&edge.chain_id]) + 1 % ROOT_HISTORY_SIZE;
            self.curr_neighbor_root_index.insert(edge.chain_id, neighbor_root_index);
            self.neighbor_roots.insert((edge.chain_id, neighbor_root_index), edge.root);
        } else {
            let edge_count = self.edges.keys().len();
            assert!(self.max_edges >= edge_count as u32 + 1, "Edge list is full");
            self.edges.insert(edge.chain_id, edge.clone());
            self.neighbor_roots.insert((edge.chain_id, 1), edge.root);
            self.curr_neighbor_root_index.insert(edge.chain_id, 1);
        }

        Ok(())
    }

    pub fn get_latest_neighbor_root(&self, chain_id: ChainId) -> anchor::Result<[u8; 32]> {
        let neighbor_root_index = self.curr_neighbor_root_index[&chain_id];
        Ok(self.neighbor_roots[&(chain_id, neighbor_root_index)])
    }

    pub fn get_latest_neighbor_edges(&self) -> Vec<Edge> {
        self.edges.values().map(|e| e.clone()).collect()
    }

    pub fn get_neighbor_roots(&self) -> Vec<[u8; 32]> {
        self.edges.values().map(|edge| edge.root).collect()
    }

    pub fn is_known_neighbor_root(&self, chain_id: ChainId, root: [u8; 32]) -> bool {
        if root == [0u8; 32] {
            return false;
        }

        let mut i = self.curr_neighbor_root_index[&chain_id];
        if root == self.neighbor_roots[&(chain_id, i)] {
            return true;
        }

        if i == 0 {
            i = ROOT_HISTORY_SIZE;
        }

        i = i - 1;
        while i != self.curr_neighbor_root_index[&chain_id] {
            if root == self.neighbor_roots[&(chain_id, i)] {
                return true;
            }

            if i == 0 {
                i = ROOT_HISTORY_SIZE;
            }

            if root == self.neighbor_roots[&(chain_id, i)] {
                return true;
            }
        }

        false
    }

    pub fn is_valid_neighbor_roots(&self, roots: &[[u8; 32]]) -> bool {
        assert!(roots.len() == self.max_edges as usize, "Incorrect roots length");
        for (i, edge) in self.edges.values().enumerate() {
            if !self.is_known_neighbor_root(edge.chain_id, roots[i]) {
                return false;
            }
        }
        return true;
    }
}
