pub struct MerkleTree {
    nodes   : Vec<u64>,
    values  : Vec<u64>
}

impl MerkleTree {

    pub fn new(leaves: Vec<u64>, hash: fn(&[u64], &mut [u64])) -> MerkleTree {

        let nodes = build_merkle_nodes(&leaves, hash);
        return MerkleTree {
            values  : leaves,
            nodes   : nodes
        };
    }

    pub fn root(&self) -> &[u64] {
        return &self.nodes[4..8];
    }

    pub fn prove(&self, index: usize) -> Vec<u64> {

        let i = index * 4;
        let mut proof = Vec::new();
        proof.push(self.values[i]);
        proof.push(self.values[i + 1]);
        proof.push(self.values[i + 2]);
        proof.push(self.values[i + 3]);

        let i = (index ^ 1) * 4;
        proof.push(self.values[i]);
        proof.push(self.values[i + 1]);
        proof.push(self.values[i + 2]);
        proof.push(self.values[i + 3]);

        let mut index = (index + self.nodes.len() / 4) >> 1;
        while index > 1 {
            let i = (index ^ 1) * 4;
            proof.push(self.nodes[i]);
            proof.push(self.nodes[i + 1]);
            proof.push(self.nodes[i + 2]);
            proof.push(self.nodes[i + 3]);
            index = index >> 1;
        }

        return proof;
    }

    pub fn prove_batch(&self, indexes: &[u64]) {

    }


    pub fn verify(root: &[u64], index: usize, proof: &[u64], hash: fn(&[u64], &mut [u64])) -> bool {
        let mut buf = [0u64; 8];
        let mut v = [0u64; 4];

        let r = (index & 1) * 4;
        &buf[0..4].copy_from_slice(&proof[r..(r + 4)]);
        let r = 4 - r;
        &buf[4..8].copy_from_slice(&proof[r..(r + 4)]);
        hash(&buf, &mut v);
        //println!("buf: {:?}, v: {:?}", &buf, &v);

        let mut index = (index + usize::pow(2, (proof.len() / 4 - 1) as u32)) >> 1;
        for i in 2..(proof.len() / 4) {
            //println!("index: {}, i: {}", index, i);
            if index & 1 == 0 {
                &buf[0..4].copy_from_slice(&v);
                &buf[4..8].copy_from_slice(&proof[(i * 4)..(i * 4 + 4)]);
            }
            else {
                &buf[0..4].copy_from_slice(&proof[(i * 4)..(i * 4 + 4)]);
                &buf[4..8].copy_from_slice(&v);
            }
            hash(&buf, &mut v);
            //println!("buf: {:?}, v: {:?}", &buf, &v);
            index = index >> 1;
        }

        return v == root;
    }

    pub fn verify_batch(root: u64, indexes: &[usize], proof: &[u64]) {
        
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn build_merkle_nodes(leaves: &[u64], hash: fn(&[u64], &mut [u64])) -> Vec<u64> {
    let n = leaves.len();
    let mut nodes = vec![0u64; n];

    // build first row of internal nodes (parents of leaves)
    for (i, j) in (0..n).step_by(8).zip(((n / 2)..n).step_by(4)) {
        hash(&leaves[i..(i + 8)], &mut nodes[j..(j + 4)]);
    }

    // calculate all other tree nodes
    for i in (1..(n / 2)).rev().skip(3).step_by(4) {
        let (parents, nodes) = nodes.split_at_mut(i * 2);
        hash(&nodes[..8], &mut parents[i..(i + 4)]);
    }

    return nodes;
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use crate::hash;

    #[test]
    fn new_merkle_tree() {
        let leaves = vec![
            6240958401110583462u64,  7913251457734141410, 10424272014552449446,  8926189284258310218,
              16554193988646091251, 18107256576288978408,  9223357806195242659,  7591105067405469359,
              11143668108497789195,  3289331174328174429, 18085733244798495096, 16874288619384630339,
              13458213771757530415, 15574026171644776407,  2236303685881236230, 16652047415881651529
        ];
        let tree = super::MerkleTree::new(leaves, hash::poseidon);
        let root = [5235193944924908127, 18013308191860768494, 18032205349443695315, 17970675247944706304];
        assert_eq!(root, tree.root());

        let leaves = vec![
            10241768711231905139u64,  9543515656056738355,  3787122002184510141,  9354315911492805116,
               14373792471285313076, 10259803863341799909,  4361913119464376502, 14664313136545201958,
               10131098303839284098,  5921316728206729490, 10334290713044556732,  8643164606753777491,
                3453858615599341263, 17558389957719367849,  9827054574735249697,  8012452355193068045,
                9196785718850699443,  6184806869699853092,  1586592971438511472,   555830527090219830,
                9952908082911899749,  3740909091289176615,   284496432800007785, 12636108119248205469,
               15468185072990248985,  9202716477534013353, 15320321401254534633,  9244660312647244009,
               13492130182068317175, 11411250703184174957,  5614217056664461616, 12322142689514354888
        ];
        let tree = super::MerkleTree::new(leaves, hash::poseidon);
        let root = [8660861239908155826, 6490529732357677518, 5539604525820699984, 17472140730726223769];
        assert_eq!(root, tree.root());
    }
}