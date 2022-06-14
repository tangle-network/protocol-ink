use super::mixer::{Result, ROOT_HISTORY_SIZE};
use super::*;
#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
use ink_storage::traits::{SpreadAllocate, SpreadLayout};
use ink_storage::Mapping;
use poseidon::PoseidonRef;

use ink_prelude::vec;

#[derive(Default, Debug, SpreadLayout, SpreadAllocate)]
#[cfg_attr(feature = "std", derive(StorageLayout))]
pub struct MerkleTree {
    pub levels: u32,
    pub current_root_index: u32,
    pub next_index: u32,
    pub filled_subtrees: Mapping<u32, [u8; 32]>,
    pub roots: Mapping<u32, [u8; 32]>,
}

impl MerkleTree {
    fn hash_left_right(
        &self,
        hasher: PoseidonRef,
        left: [u8; 32],
        right: [u8; 32],
    ) -> Result<[u8; 32]> {
        let inputs = vec![left, right];
        hasher.hash(inputs).map_err(|_| mixer::Error::HashError)
    }

    pub fn insert(&mut self, hasher: PoseidonRef, leaf: [u8; 32]) -> Result<u32> {
        let next_index = self.next_index;
        assert!(
            next_index != 2u32.pow(self.levels as u32),
            "Merkle tree is full"
        );

        let mut current_index = next_index;
        let mut current_level_hash = leaf;
        let mut left: [u8; 32];
        let mut right: [u8; 32];

        for i in 0..self.levels {
            let message = ink_prelude::format!("loop index in merkle tree insertion is {:?}", i);
            ink_env::debug_println!("{}", &message);

            if current_index % 2 == 0 {
                left = current_level_hash;
                right = zeroes::zeroes(i);
                self.filled_subtrees.insert(i, &current_level_hash);
            } else {
                left = self.filled_subtrees.get(&i).unwrap_or_default();
                right = current_level_hash;
            }

            current_level_hash = self.hash_left_right(hasher.clone(), left, right)?;
            current_index = current_index / 2;
        }

        let new_root_index = (self.current_root_index + 1) % ROOT_HISTORY_SIZE;
        self.current_root_index = new_root_index;
        self.roots.insert(new_root_index, &current_level_hash);
        self.next_index = next_index + 1;
        Ok(next_index)
    }

    pub fn is_known_root(&self, root: [u8; 32]) -> bool {
        if root == [0u8; 32] {
            return false;
        }

        let mut i = self.current_root_index;
        for _ in 0..ROOT_HISTORY_SIZE {
            if let Some(r) = self.roots.get(&i) {
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
}
