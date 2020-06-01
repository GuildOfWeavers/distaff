use crate::math::{ FiniteField };
use crate::stark::{ MAX_INPUTS };

#[derive(Clone)]
pub struct ProgramInputs<T: FiniteField> {
    public: Vec<T>,
    secret: [Vec<T>; 2],
}

impl <T> ProgramInputs<T>
    where T: FiniteField
{
    pub fn new(public: &[T], secret_a: &[T], secret_b: &[T]) -> ProgramInputs<T> {

        assert!(public.len() <= MAX_INPUTS,
            "expected no more than {} public inputs, but received {}",
            MAX_INPUTS,
            public.len());
        assert!(secret_a.len() >= secret_b.len(), 
            "number of primary secret inputs cannot be smaller than the number of secondary secret inputs");

        return ProgramInputs {
            public  : public.to_vec(),
            secret  : [secret_a.to_vec(), secret_b.to_vec()]
        };
    }

    pub fn from_public(public: &[T]) -> ProgramInputs<T> {
        return ProgramInputs {
            public: public.to_vec(),
            secret: [vec![], vec![]]
        };
    }

    pub fn get_public_inputs(&self) -> &[T] {
        return &self.public;
    }

    pub fn get_secret_inputs(&self) -> &[Vec<T>; 2] {
        return &self.secret;
    }
}