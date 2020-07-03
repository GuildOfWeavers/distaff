use crate::{ MAX_PUBLIC_INPUTS };

#[derive(Clone, Debug)]
pub struct ProgramInputs {
    public: Vec<u128>,
    secret: [Vec<u128>; 2],
}

impl ProgramInputs {

    /// Returns `ProgramInputs` initialized with the provided public and secret inputs.
    pub fn new(public: &[u128], secret_a: &[u128], secret_b: &[u128]) -> ProgramInputs {

        assert!(public.len() <= MAX_PUBLIC_INPUTS,
            "expected no more than {} public inputs, but received {}",
            MAX_PUBLIC_INPUTS,
            public.len());
        assert!(secret_a.len() >= secret_b.len(), 
            "number of primary secret inputs cannot be smaller than the number of secondary secret inputs");

        return ProgramInputs {
            public  : public.to_vec(),
            secret  : [secret_a.to_vec(), secret_b.to_vec()]
        };
    }

    /// Returns `ProgramInputs` with public and secret input tapes set to empty vectors.
    pub fn none() -> ProgramInputs {
        return ProgramInputs {
            public  : Vec::new(),
            secret  : [Vec::new(), Vec::new()],
        };
    }

    /// Returns `ProgramInputs` initialized with the provided public inputs and secret
    /// input tapes set to empty vectors.
    pub fn from_public(public: &[u128]) -> ProgramInputs {
        return ProgramInputs {
            public: public.to_vec(),
            secret: [vec![], vec![]]
        };
    }

    pub fn get_public_inputs(&self) -> &[u128] {
        return &self.public;
    }

    pub fn get_secret_inputs(&self) -> &[Vec<u128>; 2] {
        return &self.secret;
    }
}