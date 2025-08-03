pub mod opcodes {
    pub const SLOAD: u8 = 0x01;
    pub const SSTORE: u8 = 0x02;
    pub const PUSH: u8 = 0x03;
    pub const ADD: u8 = 0x04;
    pub const SUB: u8 = 0x05;
    pub const STOP: u8 = 0xFF;
}
