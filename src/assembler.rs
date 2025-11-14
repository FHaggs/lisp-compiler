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

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
// Intel style partial registers
pub enum PartialRegister {
    Al = 0,
    Cl,
    Dl,
    Bl,
    Ah,
    Ch,
    Dh,
    Bh,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
// Intel style partial registers
pub enum SetccConditions {
    Overflow = 0,     // O
    NotOverflow = 1,  // NO
    Below = 2,        // B, C, NAE
    AboveOrEqual = 3, // AE, NB, NC
    Equal = 4,        // E, Z
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
    pub fn add_reg_imm32(&mut self, dst: Register, src: i32) -> &mut Self {
        self.code.push(REX_W_PREFIX);
        self.code.push(0x81); // Opcode for immediate arithmetic
        self.code.push(0xc0 + dst as u8); // ModR/M: /0 (add)
        self.code.extend_from_slice(&(src as u32).to_le_bytes());
        self
    }
    pub fn sub_reg_imm32(&mut self, dst: Register, src: i32) -> &mut Self {
        self.code.push(REX_W_PREFIX);
        self.code.push(0x81); // Opcode for immediate arithmetic
        self.code.push(0xe8 + dst as u8); // ModR/M: /5 (sub)
        self.code.extend_from_slice(&(src as u32).to_le_bytes());
        self
    }
    pub fn shl_reg_imm8(&mut self, dst: Register, imm8: u8) -> &mut Self {
        self.code.push(REX_W_PREFIX); // 0x48
        self.code.push(0xC1); // Opcode for SHL r/m64, imm8
        self.code.push(0xE0 + dst as u8); // ModR/M: mod=11, reg=100 (/4), r/m=dst
        self.code.push(imm8); // Immediate value
        self
    }

    pub fn or_reg_imm8(&mut self, dst: Register, imm8: u8) -> &mut Self {
        self.code.push(REX_W_PREFIX); // 0x48 → use 64-bit operands
        self.code.push(0x83); // Opcode for arithmetic with imm8 (sign-extended)
        self.code.push(0xC8 + dst as u8); // ModR/M: mod=11, reg=001 (/1 = OR), r/m=dst
        self.code.push(imm8); // Immediate 8-bit value
        self
    }
    pub fn and_reg_imm8(&mut self, dst: Register, imm8: u8) -> &mut Self {
        self.code.push(REX_W_PREFIX); // 0x48 → use 64-bit operands
        self.code.push(0x83); // Opcode for arithmetic with imm8 (sign-extended)
        self.code.push(0xE0 + dst as u8); // ModR/M: mod=11, reg=100 (/4 = AND), r/m=dst
        self.code.push(imm8); // Immediate 8-bit value
        self
    }
    pub fn cmp_reg_imm32(&mut self, dst: Register, imm32: u32) -> &mut Self {
        self.code.push(REX_W_PREFIX); // 0x48 → use 64-bit operands
        self.code.push(0x81); // Opcode for immediate arithmetic
        self.code.push(0xf8 + dst as u8); // ModR/M: mod=11, reg=111 (/7 = CMP), r/m=dst
        self.code.extend_from_slice(&imm32.to_le_bytes());
        self
    }
    pub fn setcc_imm8(&mut self, cond: SetccConditions, partial: PartialRegister) -> &mut Self {
        self.code.push(0x0f);
        self.code.push(0x90 + cond as u8);
        self.code.push(0xc0 + partial as u8);
        self
    }

    /// Emits a `ret` instruction.
    pub fn ret(&mut self) -> &mut Self {
        self.code.push(0xc3);
        self // Return `&mut Self` to allow chaining
    }
}
