pub struct AssemblyError {
    message : String,
    step    : usize,
    op      : String
}

impl AssemblyError {

    /*
    pub fn invalid_op(op: &[&str]) -> AssemblyError {
        return AssemblyError {
            message: format!("")
        };
    }
    */

    pub fn missing_param(op: &[&str], step: usize) -> AssemblyError {
        return AssemblyError {
            message : format!("malformed operation {}: parameter is missing", op[0]),
            step    : step,
            op      : op.join("."),
        };
    }

    pub fn extra_param(op: &[&str], step: usize) -> AssemblyError {
        return AssemblyError {
            message : format!("malformed operation {}: too many parameters provided", op[0]),
            step    : step,
            op      : op.join("."),
        };
    }

    pub fn invalid_param(op: &[&str], step: usize) -> AssemblyError {
        return AssemblyError {
            message : format!("malformed operation {}: parameter '{}' is invalid", op[0], op[1]),
            step    : step,
            op      : op.join("."),
        };
    }

    pub fn invalid_param_reason(op: &[&str], step: usize, reason: String) -> AssemblyError {
        return AssemblyError {
            message : format!("malformed operation {}: {}", op[0], reason),
            step    : step,
            op      : op.join("."),
        };
    }

    pub fn message(&self) -> &String {
        return &self.message;
    }

    pub fn operation(&self) -> &String {
        return &self.op;
    }

    pub fn step(&self) -> usize {
        return self.step;
    }
}

impl std::fmt::Debug for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "assembly error at {}: {}", self.step, self.message)
    }
}

impl std::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "assembly error at {}: {}", self.step, self.message)
    }
}