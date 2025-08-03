pub mod opcodes {
    pub const SLOAD: u8 = 0x01;
    pub const SSTORE: u8 = 0x02;
    pub const PUSH: u8 = 0x03;
    pub const ADD: u8 = 0x04;
    pub const SUB: u8 = 0x05;
    pub const MUL: u8 = 0x06;
    pub const DIV: u8 = 0x07;
    pub const MOD: u8 = 0x08;
    pub const EQ: u8 = 0x09;
    pub const LT: u8 = 0x0A;
    pub const GT: u8 = 0x0B;
    pub const AND: u8 = 0x0C;
    pub const OR: u8 = 0x0D;
    pub const XOR: u8 = 0x0E;
    pub const DUP: u8 = 0x0F;
    pub const SWAP: u8 = 0x10;
    pub const STOP: u8 = 0xFF;
}
