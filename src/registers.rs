#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Instruction {
    Write = 0x02,
    Read = 0x03,
    BitMod = 0x05,
    LoadTx0 = 0x40,
    LoadTx1 = 0x42,
    LoadTx2 = 0x44,
    RtsTx0 = 0x81,
    RtsTx1 = 0x82,
    RtsTx2 = 0x84,
    RtsAll = 0x87,
    ReadRx0 = 0x90,
    ReadRx1 = 0x94,
    ReadStatus = 0xA0,
    RxStatus = 0xB0,
    Reset = 0xC0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    // Filters
    RxF0SidH = 0x00,
    RxF0SidL = 0x01,
    RxF0Eid8 = 0x02,
    RxF0Eid0 = 0x03,

    RxF1SidH = 0x04,
    RxF1SidL = 0x05,
    RxF1Eid8 = 0x06,
    RxF1Eid0 = 0x07,

    RxF2SidH = 0x08,
    RxF2SidL = 0x09,
    RxF2Eid8 = 0x0A,
    RxF2Eid0 = 0x0B,

    // Control Registers
    BfpCtrl = 0x0C,
    TxRtsCtrl = 0x0D,
    CanStat = 0x0E,
    CanCtrl = 0x0F,

    RxF3SidH = 0x10,
    RxF3SidL = 0x11,
    RxF3Eid8 = 0x12,
    RxF3Eid0 = 0x13,

    RxF4SidH = 0x14,
    RxF4SidL = 0x15,
    RxF4Eid8 = 0x16,
    RxF4Eid0 = 0x17,

    RxF5SidH = 0x18,
    RxF5SidL = 0x19,
    RxF5Eid8 = 0x1A,
    RxF5Eid0 = 0x1B,

    // Error and Configuration Registers
    Tec = 0x1C,
    Rec = 0x1D,

    RxM0SidH = 0x20,
    RxM0SidL = 0x21,
    RxM0Eid8 = 0x22,
    RxM0Eid0 = 0x23,

    RxM1SidH = 0x24,
    RxM1SidL = 0x25,
    RxM1Eid8 = 0x26,
    RxM1Eid0 = 0x27,

    Cnf3 = 0x28,
    Cnf2 = 0x29,
    Cnf1 = 0x2A,
    CanInte = 0x2B,
    CanIntf = 0x2C,
    Eflg = 0x2D,

    // TX Buffer 0
    TxB0Ctrl = 0x30,
    TxB0SidH = 0x31,
    TxB0SidL = 0x32,
    TxB0Eid8 = 0x33,
    TxB0Eid0 = 0x34,
    TxB0Dlc = 0x35,
    TxB0D0 = 0x36,
    TxB0D1 = 0x37,
    TxB0D2 = 0x38,
    TxB0D3 = 0x39,
    TxB0D4 = 0x3A,
    TxB0D5 = 0x3B,
    TxB0D6 = 0x3C,
    TxB0D7 = 0x3D,

    // TX Buffer 1
    TxB1Ctrl = 0x40,
    TxB1SidH = 0x41,
    TxB1SidL = 0x42,
    TxB1Eid8 = 0x43,
    TxB1Eid0 = 0x44,
    TxB1Dlc = 0x45,
    TxB1D0 = 0x46,
    TxB1D1 = 0x47,
    TxB1D2 = 0x48,
    TxB1D3 = 0x49,
    TxB1D4 = 0x4A,
    TxB1D5 = 0x4B,
    TxB1D6 = 0x4C,
    TxB1D7 = 0x4D,

    // TX Buffer 2
    TxB2Ctrl = 0x50,
    TxB2SidH = 0x51,
    TxB2SidL = 0x52,
    TxB2Eid8 = 0x53,
    TxB2Eid0 = 0x54,
    TxB2Dlc = 0x55,
    TxB2D0 = 0x56,
    TxB2D1 = 0x57,
    TxB2D2 = 0x58,
    TxB2D3 = 0x59,
    TxB2D4 = 0x5A,
    TxB2D5 = 0x5B,
    TxB2D6 = 0x5C,
    TxB2D7 = 0x5D,

    // RX Buffer 0
    RxB0Ctrl = 0x60,
    RxB0SidH = 0x61,
    RxB0SidL = 0x62,
    RxB0Eid8 = 0x63,
    RxB0Eid0 = 0x64,
    RxB0Dlc = 0x65,
    RxB0D0 = 0x66,
    RxB0D1 = 0x67,
    RxB0D2 = 0x68,
    RxB0D3 = 0x69,
    RxB0D4 = 0x6A,
    RxB0D5 = 0x6B,
    RxB0D6 = 0x6C,
    RxB0D7 = 0x6D,

    // RX Buffer 1
    RxB1Ctrl = 0x70,
    RxB1SidH = 0x71,
    RxB1SidL = 0x72,
    RxB1Eid8 = 0x73,
    RxB1Eid0 = 0x74,
    RxB1Dlc = 0x75,
    RxB1D0 = 0x76,
    RxB1D1 = 0x77,
    RxB1D2 = 0x78,
    RxB1D3 = 0x79,
    RxB1D4 = 0x7A,
    RxB1D5 = 0x7B,
    RxB1D6 = 0x7C,
    RxB1D7 = 0x7D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OperationMode {
    Normal = 0x00,
    Sleep = 0x20,
    Loopback = 0x40,
    ListenOnly = 0x60,
    Config = 0x80,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_values_are_correct() {
        assert_eq!(Instruction::Write as u8, 0x02);
        assert_eq!(Instruction::Read as u8, 0x03);
        assert_eq!(Instruction::BitMod as u8, 0x05);
        assert_eq!(Instruction::LoadTx0 as u8, 0x40);
        assert_eq!(Instruction::LoadTx1 as u8, 0x42);
        assert_eq!(Instruction::LoadTx2 as u8, 0x44);
        assert_eq!(Instruction::RtsTx0 as u8, 0x81);
        assert_eq!(Instruction::RtsTx1 as u8, 0x82);
        assert_eq!(Instruction::RtsTx2 as u8, 0x84);
        assert_eq!(Instruction::RtsAll as u8, 0x87);
        assert_eq!(Instruction::ReadRx0 as u8, 0x90);
        assert_eq!(Instruction::ReadRx1 as u8, 0x94);
        assert_eq!(Instruction::ReadStatus as u8, 0xA0);
        assert_eq!(Instruction::RxStatus as u8, 0xB0);
        assert_eq!(Instruction::Reset as u8, 0xC0);
    }

    #[test]
    fn register_values_are_correct() {
        assert_eq!(Register::RxF0SidH as u8, 0x00);
        assert_eq!(Register::RxF0SidL as u8, 0x01);
        assert_eq!(Register::RxF0Eid8 as u8, 0x02);
        assert_eq!(Register::RxF0Eid0 as u8, 0x03);
        assert_eq!(Register::RxF1SidH as u8, 0x04);
        assert_eq!(Register::RxF2SidH as u8, 0x08);
        assert_eq!(Register::RxF3SidH as u8, 0x10);
        assert_eq!(Register::RxF4SidH as u8, 0x14);
        assert_eq!(Register::RxF5SidH as u8, 0x18);
        assert_eq!(Register::RxM0SidH as u8, 0x20);
        assert_eq!(Register::RxM1SidH as u8, 0x24);
        assert_eq!(Register::CanStat as u8, 0x0E);
        assert_eq!(Register::CanCtrl as u8, 0x0F);
        assert_eq!(Register::Tec as u8, 0x1C);
        assert_eq!(Register::Rec as u8, 0x1D);
        assert_eq!(Register::Cnf3 as u8, 0x28);
        assert_eq!(Register::Cnf2 as u8, 0x29);
        assert_eq!(Register::Cnf1 as u8, 0x2A);
        assert_eq!(Register::CanInte as u8, 0x2B);
        assert_eq!(Register::CanIntf as u8, 0x2C);
        assert_eq!(Register::Eflg as u8, 0x2D);
        assert_eq!(Register::TxB0Ctrl as u8, 0x30);
        assert_eq!(Register::TxB0SidH as u8, 0x31);
        assert_eq!(Register::TxB1Ctrl as u8, 0x40);
        assert_eq!(Register::TxB1SidH as u8, 0x41);
        assert_eq!(Register::TxB2Ctrl as u8, 0x50);
        assert_eq!(Register::TxB2SidH as u8, 0x51);
        assert_eq!(Register::RxB0Ctrl as u8, 0x60);
        assert_eq!(Register::RxB0SidH as u8, 0x61);
        assert_eq!(Register::RxB1Ctrl as u8, 0x70);
        assert_eq!(Register::RxB1SidH as u8, 0x71);
    }

    #[test]
    fn operation_mode_values_are_correct() {
        assert_eq!(OperationMode::Normal as u8, 0x00);
        assert_eq!(OperationMode::Sleep as u8, 0x20);
        assert_eq!(OperationMode::Loopback as u8, 0x40);
        assert_eq!(OperationMode::ListenOnly as u8, 0x60);
        assert_eq!(OperationMode::Config as u8, 0x80);
    }

    #[test]
    fn instruction_repr_is_u8() {
        let v: u8 = Instruction::Reset as u8;
        assert_eq!(v, 0xC0);
    }

    #[test]
    fn register_repr_is_u8() {
        let v: u8 = Register::CanCtrl as u8;
        assert_eq!(v, 0x0F);
    }

    #[test]
    fn operation_mode_repr_is_u8() {
        let v: u8 = OperationMode::Loopback as u8;
        assert_eq!(v, 0x40);
    }

    #[test]
    fn instruction_enum_is_exhaustive() {
        let variants = [
            Instruction::Write,
            Instruction::Read,
            Instruction::BitMod,
            Instruction::LoadTx0,
            Instruction::LoadTx1,
            Instruction::LoadTx2,
            Instruction::RtsTx0,
            Instruction::RtsTx1,
            Instruction::RtsTx2,
            Instruction::RtsAll,
            Instruction::ReadRx0,
            Instruction::ReadRx1,
            Instruction::ReadStatus,
            Instruction::RxStatus,
            Instruction::Reset,
        ];

        assert_eq!(variants.len(), 15);
    }

    #[test]
    fn match_on_operation_mode() {
        let mode = OperationMode::Loopback;

        let s = match mode {
            OperationMode::Normal => "normal",
            OperationMode::Sleep => "sleep",
            OperationMode::Loopback => "loopback",
            OperationMode::ListenOnly => "listen",
            OperationMode::Config => "config",
        };

        assert_eq!(s, "loopback");
    }

    #[test]
    fn operation_modes_are_unique() {
        assert_ne!(OperationMode::Normal as u8, OperationMode::Sleep as u8);
        assert_ne!(OperationMode::Sleep as u8, OperationMode::Loopback as u8);
        assert_ne!(
            OperationMode::Loopback as u8,
            OperationMode::ListenOnly as u8
        );
        assert_ne!(OperationMode::ListenOnly as u8, OperationMode::Config as u8);
    }

    #[test]
    fn filter_and_mask_register_sequence() {
        // Verify that filter and mask register constants increment sequentially as expected by write_id_registers
        assert_eq!(Register::RxF0SidL as u8, Register::RxF0SidH as u8 + 1);
        assert_eq!(Register::RxF0Eid8 as u8, Register::RxF0SidH as u8 + 2);
        assert_eq!(Register::RxF0Eid0 as u8, Register::RxF0SidH as u8 + 3);

        assert_eq!(Register::RxM0SidL as u8, Register::RxM0SidH as u8 + 1);
        assert_eq!(Register::RxM0Eid8 as u8, Register::RxM0SidH as u8 + 2);
        assert_eq!(Register::RxM0Eid0 as u8, Register::RxM0SidH as u8 + 3);
    }
}
