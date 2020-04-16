use std::slice;
use std::collections::{ HashMap, BTreeSet };
use serde::{ Serialize, Deserialize };
use crate::crypto::{ HashFunction };

// TYPES AND INTERFACES
// ================================================================================================
pub struct MerkleTree {
    nodes   : Vec<[u64; 4]>,
    values  : Vec<[u64; 4]>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMerkleProof {
    values  : Vec<[u64; 4]>,
    nodes   : Vec<Vec<[u64; 4]>>,
    depth   : u32
}

// MERKLE TREE IMPLEMENTATION
// ================================================================================================
impl MerkleTree {

    /// Creates a new merkle tree from the provide leaves and using the provided hash function.
    pub fn new(leaves: Vec<[u64; 4]>, hash: HashFunction) -> MerkleTree {
        assert!(leaves.len().is_power_of_two(), "number of leaves must be a power of 2");
        assert!(leaves.len() >= 2, "a tree must contain at least 2 leaves");

        let nodes = build_merkle_nodes(&leaves, hash);
        return MerkleTree {
            values  : leaves,
            nodes   : nodes
        };
    }

    /// Returns the root of the tree
    pub fn root(&self) -> &[u64; 4] {
        return &self.nodes[1];
    }

    /// Returns leaf nodes of the tree
    pub fn leaves(&self) -> &[[u64; 4]] {
        return &self.values;
    }

    /// Computes merkle path the given leaf index.
    pub fn prove(&self, index: usize) -> Vec<[u64; 4]> {
        assert!(index < self.values.len(), "invalid index {}", index);

        let mut proof = Vec::new();
        proof.push(self.values[index]);
        proof.push(self.values[index ^ 1]);

        let mut index = (index + self.nodes.len()) >> 1;
        while index > 1 {
            proof.push(self.nodes[index ^ 1]);
            index = index >> 1;
        }

        return proof;
    }

    /// Computes merkle paths for the provided indexes and compresses the paths into a single proof.
    pub fn prove_batch(&self, indexes: &[usize]) -> BatchMerkleProof {
        let n = self.values.len();

        let index_map = map_indexes(indexes, n);
        let indexes = normalize_indexes(indexes);
        let mut values = vec![[0, 0, 0, 0]; index_map.len()];
        let mut nodes: Vec<Vec<[u64; 4]>> = Vec::with_capacity(indexes.len());

        // populate the proof with leaf node values
        let mut next_indexes: Vec<usize> = Vec::new();
        for index in indexes {
            let v1 = self.values[index];
            let v2 = self.values[index + 1];

            // only values for indexes that were explicitly requested are included in values array
            let input_index1 = index_map.get(&index);
            let input_index2 = index_map.get(&(index + 1));
            if input_index1.is_some() {
                if input_index2.is_some() {
                    values[*input_index1.unwrap()] = v1;
                    values[*input_index2.unwrap()] = v2;
                    nodes.push(Vec::new());
                }
                else {
                    values[*input_index1.unwrap()] = v1;
                    nodes.push(vec![v2]);
                }
            }
            else {
                values[*input_index2.unwrap()] = v2;
                nodes.push(vec![v1]);
            }

            next_indexes.push((index + n) >> 1);
        }

        // add required internal nodes to the proof, skipping redundancies
        let depth = self.values.len().trailing_zeros();
        for _ in 1..depth {
            let indexes = next_indexes.clone();
            next_indexes.truncate(0);

            let mut i = 0;
            while i < indexes.len() {
                let sibling_index = indexes[i] ^ 1;
                if i + 1 < indexes.len() && indexes[i + 1] == sibling_index {
                    i += 1;
                }
                else {
                    nodes[i].push(self.nodes[sibling_index]);
                }

                // add parent index to the set of next indexes
                next_indexes.push(sibling_index >> 1);

                i += 1;
            }
        }

        return BatchMerkleProof { values, nodes, depth };
    }

    /// Checks whether the path for the specified index is valid.
    pub fn verify(root: &[u64; 4], index: usize, proof: &[[u64; 4]], hash: HashFunction) -> bool {
        let mut buf = [0u64; 8];
        let mut v = [0u64; 4];

        let r = index & 1;
        &buf[0..4].copy_from_slice(&proof[r]);
        &buf[4..8].copy_from_slice(&proof[1 - r]);
        hash(&buf, &mut v);

        let mut index = (index + usize::pow(2, (proof.len() - 1) as u32)) >> 1;
        for i in 2..proof.len() {
            if index & 1 == 0 {
                &buf[0..4].copy_from_slice(&v);
                &buf[4..8].copy_from_slice(&proof[i]);
            }
            else {
                &buf[0..4].copy_from_slice(&proof[i]);
                &buf[4..8].copy_from_slice(&v);
            }
            hash(&buf, &mut v);
            index = index >> 1;
        }

        return v == *root;
    }

    /// Checks whether the batch proof contains merkle paths for the of the specified indexes.
    pub fn verify_batch(root: &[u64; 4], indexes: &[usize], proof: &BatchMerkleProof, hash: HashFunction) -> bool {
        let mut buf = [0u64; 8];
        let mut v: HashMap<usize, [u64; 4]> = HashMap::new();

        // replace odd indexes, offset, and sort in ascending order
        let offset = usize::pow(2, proof.depth);
        let index_map = map_indexes(indexes, offset - 1);
        let indexes = normalize_indexes(indexes);
        if indexes.len() != proof.nodes.len() { return false; }

        // for each index use values to compute parent nodes
        let mut next_indexes: Vec<usize> = Vec::new();
        let mut proof_pointers: Vec<usize> = Vec::with_capacity(indexes.len());
        for (i, index) in indexes.into_iter().enumerate() {
            // copy values of leaf sibling leaf nodes into the buffer
            match index_map.get(&index) {
                Some(&index1) => {
                    if proof.values.len() <= index1 { return false }
                    &buf[0..4].copy_from_slice(&proof.values[index1]);
                    match index_map.get(&(index + 1)) {
                        Some(&index2) => {
                            if proof.values.len() <= index2 { return false }
                            &buf[4..8].copy_from_slice(&proof.values[index2]);
                            proof_pointers.push(0);
                        },
                        None => {
                            if proof.nodes[i].len() < 1 { return false }
                            &buf[4..8].copy_from_slice(&proof.nodes[i][0]);
                            proof_pointers.push(1);
                        }
                    }
                },
                None => {
                    if proof.nodes[i].len() < 1 { return false }
                    &buf[0..4].copy_from_slice(&proof.nodes[i][0]);
                    match index_map.get(&(index + 1)) {
                        Some(&index2) => {
                            if proof.values.len() <= index2 { return false }
                            &buf[4..8].copy_from_slice(&proof.values[index2]);
                        },
                        None => return false
                    }
                    proof_pointers.push(1);
                }
            }

            // hash sibling nodes into their parent
            let mut parent = [0u64; 4];
            hash(&buf, &mut parent);

            let parent_index = offset + index >> 1;
            v.insert(parent_index, parent);
            next_indexes.push(parent_index);
        }

        // iteratively move up, until we get to the root
        for _ in 1..proof.depth {
            let indexes = next_indexes.clone();
            next_indexes.truncate(0);

            let mut i = 0;
            while i < indexes.len() {
                let node_index = indexes[i];
                let sibling_index = node_index ^ 1;

                // determine the sibling
                let sibling: &[u64; 4];
                if i + 1 < indexes.len() && indexes[i + 1] == sibling_index {
                    sibling = match v.get(&sibling_index) {
                        Some(sibling) => sibling,
                        None => return false
                    };
                    i += 1;
                }
                else {
                    let pointer = proof_pointers[i];
                    if proof.nodes[i].len() <= pointer { return false }
                    sibling = &proof.nodes[i][pointer];
                    proof_pointers[i] += 1;
                }

                // get the node from the map of hashed nodes
                let node = match v.get(&node_index) {
                    Some(node) => node,
                    None => return false
                };

                // compute parent node from node and sibling
                if node_index & 1 != 0 {
                    &buf[0..4].copy_from_slice(sibling);
                    &buf[4..8].copy_from_slice(node);
                }
                else {
                    &buf[0..4].copy_from_slice(node);
                    &buf[4..8].copy_from_slice(sibling);
                }
                let mut parent = [0u64; 4];
                hash(&buf, &mut parent);

                // add the parent node to the next set of nodes
                let parent_index = node_index >> 1;
                v.insert(parent_index, parent);
                next_indexes.push(parent_index);

                i += 1;
            }
        }
     
        return *root == *v.get(&1).unwrap();
    }
}

// BATCH MERKLE PROOF IMPLEMENTATION
// ================================================================================================
impl BatchMerkleProof {

    pub fn values(&self) -> &Vec<[u64; 4]> {
        return &self.values;
    }
    
}

// HELPER FUNCTIONS
// ================================================================================================

fn build_merkle_nodes(leaves: &[[u64; 4]], hash: HashFunction) -> Vec<[u64; 4]> {
    let n = leaves.len() / 2;

    // create un-initialized array to hold all intermediate nodes
    let mut nodes: Vec<[u64; 4]> = Vec::with_capacity(2 * n);
    unsafe { nodes.set_len(2 * n); }
    nodes[0] = [0, 0, 0, 0];

    // re-interpret leaves as an array of two leaves fused together
    let two_leaves = unsafe { slice::from_raw_parts(leaves.as_ptr() as *const [u64; 8], n) };

    // build first row of internal nodes (parents of leaves)
    for (i, j) in (0..n).zip(n..nodes.len()) {
        hash(&two_leaves[i], &mut nodes[j]);
    }

    // re-interpret nodes as an array of two nodes fused together
    let two_nodes = unsafe { slice::from_raw_parts(nodes.as_ptr() as *const [u64; 8], n) };

    // calculate all other tree nodes
    for i in (1..n).rev() {
        hash(&two_nodes[i], &mut nodes[i]);
    }

    return nodes;
}

fn map_indexes(indexes: &[usize], max_valid: usize) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    for (i, index) in indexes.iter().cloned().enumerate() {
        map.insert(index, i);
        assert!(index <= max_valid, "invalid index {}", index);
    }
    assert!(indexes.len() == map.len(), "repeating indexes detected");
    return map;
}

fn normalize_indexes(indexes: &[usize]) -> Vec<usize> {
    let mut set = BTreeSet::new();
    for &index in indexes {
        set.insert(index - (index & 1));
    }
    return set.into_iter().collect();
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use crate::hash;

    static LEAVES4: [[u64; 4]; 4] = [
        [6240958401110583462u64,  7913251457734141410,  10424272014552449446,  8926189284258310218],
        [  16554193988646091251, 18107256576288978408,   9223357806195242659,  7591105067405469359],
        [  11143668108497789195,  3289331174328174429,  18085733244798495096, 16874288619384630339],
        [  13458213771757530415, 15574026171644776407,   2236303685881236230, 16652047415881651529]
    ];

    static LEAVES8: [[u64; 4]; 8] = [
        [10241768711231905139u64,  9543515656056738355,  3787122002184510141,  9354315911492805116],
        [   14373792471285313076, 10259803863341799909,  4361913119464376502, 14664313136545201958],
        [   10131098303839284098,  5921316728206729490, 10334290713044556732,  8643164606753777491],
        [    3453858615599341263, 17558389957719367849,  9827054574735249697,  8012452355193068045],
        [    9196785718850699443,  6184806869699853092,  1586592971438511472,   555830527090219830],
        [    9952908082911899749,  3740909091289176615,   284496432800007785, 12636108119248205469],
        [   15468185072990248985,  9202716477534013353, 15320321401254534633,  9244660312647244009],
        [   13492130182068317175, 11411250703184174957,  5614217056664461616, 12322142689514354888]
    ];

    #[test]
    fn new_tree() {
        let leaves = LEAVES4.to_vec();
        let tree = super::MerkleTree::new(leaves, hash::poseidon);
        let root = [5235193944924908127, 18013308191860768494, 18032205349443695315, 17970675247944706304];
        assert_eq!(&root, tree.root());

        let leaves = LEAVES8.to_vec();
        let tree = super::MerkleTree::new(leaves, hash::poseidon);
        let root = [8660861239908155826, 6490529732357677518, 5539604525820699984, 17472140730726223769];
        assert_eq!(&root, tree.root());
    }

    #[test]
    fn prove() {
        // depth 4
        let leaves = LEAVES4.to_vec();
        let tree = super::MerkleTree::new(leaves, hash::poseidon);

        let proof = vec![
            [16554193988646091251u64, 18107256576288978408,  9223357806195242659,  7591105067405469359],
            [    6240958401110583462,  7913251457734141410, 10424272014552449446,  8926189284258310218],
            [    7687062139022541075, 11330299340636142278, 12047930580301070421, 14108397826062915505]
        ];
        assert_eq!(proof, tree.prove(1));

        let proof = vec![
            [11143668108497789195u64,  3289331174328174429, 18085733244798495096, 16874288619384630339],
            [   13458213771757530415, 15574026171644776407,  2236303685881236230, 16652047415881651529],
            [   13561797516333500728,  8216566980108857093,  1571520667796023534,  7179744582708748410]
        ];
        assert_eq!(proof, tree.prove(2));

        // depth 5
        let leaves = LEAVES8.to_vec();
        let tree = super::MerkleTree::new(leaves, hash::poseidon);

        let proof = vec![
            [14373792471285313076u64, 10259803863341799909, 4361913119464376502, 14664313136545201958],
            [   10241768711231905139,  9543515656056738355, 3787122002184510141,  9354315911492805116],
            [      44908261164380711,  3641300267207966756, 9037481878828096793,  1015347137991923011],
            [   10450220324776338674,  4170064214771947868, 3580214161055284290, 10659497852322269609]
        ];
        assert_eq!(proof, tree.prove(1));

        let proof = vec![
            [15468185072990248985u64,  9202716477534013353,  15320321401254534633,  9244660312647244009],
            [   13492130182068317175, 11411250703184174957,   5614217056664461616, 12322142689514354888],
            [    4955870195980606083, 11478659773640590941,   5548116754451226534, 14304409171415235034],
            [    3026191785366862170,  5135587167437964504,  14843759496938774975,  7330364722345621919]
        ];
        assert_eq!(proof, tree.prove(6));
    }

    #[test]
    fn verify() {
        // depth 4
        let leaves = LEAVES4.to_vec();
        let tree = super::MerkleTree::new(leaves, hash::poseidon);
        let proof = tree.prove(1);
        assert_eq!(true, super::MerkleTree::verify(tree.root(), 1, &proof, hash::poseidon));

        let proof = tree.prove(2);
        assert_eq!(true, super::MerkleTree::verify(tree.root(), 2, &proof, hash::poseidon));

        // depth 5
        let leaves = LEAVES8.to_vec();
        let tree = super::MerkleTree::new(leaves, hash::poseidon);
        let proof = tree.prove(1);
        assert_eq!(true, super::MerkleTree::verify(tree.root(), 1, &proof, hash::poseidon));

        let proof = tree.prove(6);
        assert_eq!(true, super::MerkleTree::verify(tree.root(), 6, &proof, hash::poseidon));
    }

    #[test]
    fn prove_batch() {
        let leaves = LEAVES8.to_vec();
        let tree = super::MerkleTree::new(leaves, hash::poseidon);
        
        // 1 index
        let proof = tree.prove_batch(&[1]);
        let expected_values = vec![
            [14373792471285313076u64, 10259803863341799909,  4361913119464376502, 14664313136545201958]
        ];
        let expected_nodes = vec![
            vec![
                [10241768711231905139u64, 9543515656056738355, 3787122002184510141,  9354315911492805116],
                [      44908261164380711, 3641300267207966756, 9037481878828096793,  1015347137991923011],
                [   10450220324776338674, 4170064214771947868, 3580214161055284290, 10659497852322269609]
            ]
        ];
        assert_eq!(expected_values, proof.values);
        assert_eq!(expected_nodes, proof.nodes);
        assert_eq!(3, proof.depth);

        // 2 indexes
        let proof = tree.prove_batch(&[1, 2]);
        let expected_values = vec![
            [14373792471285313076u64, 10259803863341799909,  4361913119464376502, 14664313136545201958],
            [   10131098303839284098,  5921316728206729490, 10334290713044556732,  8643164606753777491]
        ];
        let expected_nodes = vec![
            vec![
                [10241768711231905139u64, 9543515656056738355, 3787122002184510141,  9354315911492805116],
                [   10450220324776338674, 4170064214771947868, 3580214161055284290, 10659497852322269609]
            ],
            vec![
                [    3453858615599341263, 17558389957719367849, 9827054574735249697, 8012452355193068045]
            ]
        ];
        assert_eq!(expected_values, proof.values);
        assert_eq!(expected_nodes, proof.nodes);
        assert_eq!(3, proof.depth);

        // 2 indexes on opposite sides
        let proof = tree.prove_batch(&[1, 6]);
        let expected_values = vec![
            [14373792471285313076u64, 10259803863341799909,  4361913119464376502, 14664313136545201958],
            [   15468185072990248985,  9202716477534013353, 15320321401254534633,  9244660312647244009]
        ];
        let expected_nodes = vec![
            vec![
                [10241768711231905139u64, 9543515656056738355, 3787122002184510141,  9354315911492805116],
                [      44908261164380711, 3641300267207966756, 9037481878828096793,  1015347137991923011]
            ],
            vec![
                [   13492130182068317175, 11411250703184174957, 5614217056664461616, 12322142689514354888],
                [    4955870195980606083, 11478659773640590941, 5548116754451226534, 14304409171415235034]
            ]
        ];
        assert_eq!(expected_values, proof.values);
        assert_eq!(expected_nodes, proof.nodes);
        assert_eq!(3, proof.depth);

        // all indexes
        let proof = tree.prove_batch(&[0, 1, 2, 3, 4, 5, 6, 7]);
        let expected_values = LEAVES8.to_vec();
        let expected_nodes: Vec<Vec<[u64; 4]>> = vec![ vec![], vec![], vec![], vec![]];
        assert_eq!(expected_values, proof.values);
        assert_eq!(expected_nodes, proof.nodes);
        assert_eq!(3, proof.depth);
    }

    #[test]
    fn verify_batch() {
        let leaves = LEAVES8.to_vec();
        let tree = super::MerkleTree::new(leaves, hash::poseidon);

        let proof = tree.prove_batch(&[1]);
        assert_eq!(true, super::MerkleTree::verify_batch(tree.root(), &[1], &proof, hash::poseidon));
        assert_eq!(false, super::MerkleTree::verify_batch(tree.root(), &[2], &proof, hash::poseidon));

        let proof = tree.prove_batch(&[1, 2]);
        assert_eq!(true, super::MerkleTree::verify_batch(tree.root(), &[1, 2], &proof, hash::poseidon));
        assert_eq!(false, super::MerkleTree::verify_batch(tree.root(), &[1], &proof, hash::poseidon));
        assert_eq!(false, super::MerkleTree::verify_batch(tree.root(), &[1, 3], &proof, hash::poseidon));
        assert_eq!(false, super::MerkleTree::verify_batch(tree.root(), &[1, 2, 3], &proof, hash::poseidon));

        let proof = tree.prove_batch(&[1, 6]);
        assert_eq!(true, super::MerkleTree::verify_batch(tree.root(), &[1, 6], &proof, hash::poseidon));

        let proof = tree.prove_batch(&[1, 3, 6]);
        assert_eq!(true, super::MerkleTree::verify_batch(tree.root(), &[1, 3, 6], &proof, hash::poseidon));

        let proof = tree.prove_batch(&[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(true, super::MerkleTree::verify_batch(tree.root(), &[0, 1, 2, 3, 4, 5, 6, 7], &proof, hash::poseidon));
    }
}