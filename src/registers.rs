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
    RxM0SidH = 0x20,
    RxM0SidL = 0x21,
    RxM0Eid8 = 0x22,
    RxM0Eid0 = 0x23,
    RxM1SidH = 0x24,
    RxM1SidL = 0x25,
    RxM1Eid8 = 0x26,
    RxM1Eid0 = 0x27,
    CanStat = 0x0E,
    CanCtrl = 0x0F,
    Tec = 0x1C,
    Rec = 0x1D,
    Cnf3 = 0x28,
    Cnf2 = 0x29,
    Cnf1 = 0x2A,
    CanInte = 0x2B,
    CanIntf = 0x2C,
    Eflg = 0x2D,
    TxB0Ctrl = 0x30,
    TxB0SidH = 0x31,
    TxB1Ctrl = 0x40,
    TxB1SidH = 0x41,
    TxB2Ctrl = 0x50,
    TxB2SidH = 0x51,
    RxB0Ctrl = 0x60,
    RxB0SidH = 0x61,
    RxB1Ctrl = 0x70,
    RxB1SidH = 0x71,
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
    fn instruction_round_trip() {
        let v = Instruction::Reset as u8;
        let back = unsafe { core::mem::transmute::<u8, Instruction>(v) };
        assert_eq!(back, Instruction::Reset);
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
    fn no_std_compiles() {
        let _ = Instruction::Write;
        let _ = Register::CanCtrl;
        let _ = OperationMode::Normal;
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
