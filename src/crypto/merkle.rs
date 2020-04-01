use std::slice;

// TYPES AND INTERFACES
// ================================================================================================
pub struct MerkleTree {
    nodes   : Vec<[u64; 4]>,
    values  : Vec<[u64; 4]>
}

type HashFunction = fn(&[u64], &mut [u64]);

// METHOD DEFINITIONS
// ================================================================================================
impl MerkleTree {

    pub fn new(leaves: Vec<[u64; 4]>, hash: HashFunction) -> MerkleTree {
        assert!(leaves.len().is_power_of_two(), "number of leaves must be a power of 2");
        assert!(leaves.len() >= 2, "a tree must consist of at least 2 leaves");

        let nodes = build_merkle_nodes(&leaves, hash);
        return MerkleTree {
            values  : leaves,
            nodes   : nodes
        };
    }

    pub fn root(&self) -> &[u64; 4] {
        return &self.nodes[1];
    }

    pub fn prove(&self, index: usize) -> Vec<[u64; 4]> {

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

    pub fn prove_batch(&self, indexes: &[u64]) {

    }


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

    pub fn verify_batch(root: u64, indexes: &[usize], proof: &[u64]) {
        
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
}