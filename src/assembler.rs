#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Register {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rbx = 3,
    Rsp = 4,
    Rbp = 5,
    Rsi = 6,
    Rdi = 7,
}

const REX_W_PREFIX: u8 = 0x48;
pub struct Assembler {
    code: Vec<u8>,
}

impl Assembler {
    pub fn new() -> Self {
        Assembler { code: Vec::new() }
    }

    /// Consumes the assembler and returns the raw machine code bytes. IT SHOULD< FOR NOW USE JUST A REF
    pub fn finalize(&self) -> &Vec<u8> {
        &self.code
    }

    /// Emits a 64-bit "move register, immediate" instruction.
    /// Example: `mov rax, 42`
    pub fn mov_reg_imm32(&mut self, dst: Register, src: i32) -> &mut Self {
        self.code.push(REX_W_PREFIX);
        self.code.push(0xc7);
        self.code.push(0xc0 + dst as u8);
        self.code.extend_from_slice(&(src as u32).to_le_bytes());

        self // Return `&mut Self` to allow chaining
    }

    /// Emits a `ret` instruction.
    pub fn ret(&mut self) -> &mut Self {
        self.code.push(0xc3);
        self // Return `&mut Self` to allow chaining
    }
}
