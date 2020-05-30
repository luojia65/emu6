pub struct IntRegister {
    registers: [u64; 32],
}

pub struct Csr {
    inner: [u64; 4096],
}

