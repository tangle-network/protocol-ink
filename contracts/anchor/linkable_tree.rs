use super::*;
use ink_storage::Mapping;
use ink_prelude::vec::Vec;
use ink_storage::traits::{SpreadLayout, PackedLayout};
#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
use ink_storage::traits::{PackedLayout, SpreadLayout};

use ink_storage::traits::SpreadAllocate;

pub type ChainId = u64;

#[derive(Default, Debug, SpreadLayout, SpreadAllocate)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct LinkableMerkleTree {
    pub max_edges: u32,
    pub chain_id_list: Vec<ChainId>,
    pub edges: Mapping<ChainId, Edge>,
    pub curr_neighbor_root_index: Mapping<ChainId, u32>,
    pub neighbor_roots: Mapping<(ChainId, u32), [u8; 32]>,
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
        self.edges.get(&chain_id).is_some()
    }

    pub fn update_edge(&mut self, edge: Edge) -> anchor::Result<()> {
        if self.has_edge(edge.chain_id) {
            assert!(
                edge.latest_leaf_index < self.edges.get(&edge.chain_id).unwrap_or_default().latest_leaf_index + 65_536,
                "latest leaf index should be greater than the previous one"
            );
            self.edges.insert(edge.chain_id, &edge);
            let curr_neighbor_root_index = self.curr_neighbor_root_index.get(&edge.chain_id).unwrap_or_default();
            let neighbor_root_index = curr_neighbor_root_index + 1 % ROOT_HISTORY_SIZE;
            self.curr_neighbor_root_index.insert(edge.chain_id, &neighbor_root_index);
            self.neighbor_roots.insert((edge.chain_id, neighbor_root_index), &edge.root);
        } else {
            let edge_count = self.chain_id_list.len() as u32;
            assert!(self.max_edges >= edge_count as u32 + 1, "Edge list is full");
            self.edges.insert(edge.chain_id, &edge);
            self.neighbor_roots.insert((edge.chain_id, 1), &edge.root);
            self.curr_neighbor_root_index.insert(edge.chain_id, &1);
            self.chain_id_list.push(edge.chain_id);
        }

        Ok(())
    }

    pub fn get_latest_neighbor_root(&self, chain_id: ChainId) -> anchor::Result<[u8; 32]> {
        let neighbor_root_index = self.curr_neighbor_root_index.get(&chain_id).ok_or(anchor::Error::NotInitialized)?;
        let latest_neighbor_root = self.neighbor_roots.get(&(chain_id, neighbor_root_index)).ok_or(anchor::Error::NotInitialized)?;
        Ok(latest_neighbor_root)
    }

    pub fn get_latest_neighbor_edges(&self) -> Vec<Edge> {
        self.chain_id_list.iter().map(|c_id| self.edges.get(c_id).unwrap_or_default()).collect()
    }

    pub fn get_neighbor_roots(&self) -> Vec<[u8; 32]> {
        self.chain_id_list.iter().map(|c_id| self.edges.get(c_id).unwrap_or_default().root).collect()
    }

    pub fn is_known_neighbor_root(&self, chain_id: ChainId, root: [u8; 32]) -> bool {
        if root == [0u8; 32] {
            return false;
        }

        let mut i = self.curr_neighbor_root_index.get(&chain_id).unwrap_or_default();
        for _ in 0..ROOT_HISTORY_SIZE {
            if let Some(r) = self.neighbor_roots.get(&(chain_id, i)) {
                if r == root {
                    return true;
                }
    
                if i == 0 {
                    i = ROOT_HISTORY_SIZE - 1;
                } else {
                    i -= 1;
                }
            }
        }

        false
    }

    pub fn is_valid_neighbor_roots(&self, roots: &[[u8; 32]]) -> bool {
        assert!(roots.len() == self.max_edges as usize, "Incorrect roots length");
        for (i, edge) in self.chain_id_list.iter().map(|c_id| self.edges.get(c_id).unwrap_or_default()).enumerate() {
            if !self.is_known_neighbor_root(edge.chain_id, roots[i]) {
                return false;
            }
        }
        return true;
    }
}
