mod common;

use common::harness::TestDevice;
use mcp2515_async::registers::{Instruction, OperationMode, Register};
use mcp2515_async::BitrateConfig;

#[tokio::test]
async fn reset_sends_correct_instruction() {
    let mut t = TestDevice::new();

    t.dev.reset().await.unwrap();

    let writes = t.writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0], vec![Instruction::Reset as u8]);
}

#[tokio::test]
async fn set_mode_sends_bitmod() {
    let mut t = TestDevice::new();

    t.dev.set_mode(OperationMode::Normal).await.unwrap();

    assert_eq!(t.spi.writes().len(), 1);
    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::CanCtrl as u8,
            0xE0,
            OperationMode::Normal as u8
        ]
    );
}

#[tokio::test]
async fn read_register_returns_third_byte() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0xAB]);

    let val = t.dev.read_register(Register::CanCtrl).await.unwrap();

    assert_eq!(val, 0xAB);
    assert_eq!(t.spi.transfers().len(), 1);
    assert_eq!(
        t.spi.transfers()[0].1,
        vec![Instruction::Read as u8, Register::CanCtrl as u8, 0]
    );
}

#[tokio::test]
async fn set_register_sends_write() {
    let mut t = TestDevice::new();

    t.dev.set_register(Register::CanCtrl, 0x55).await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![Instruction::Write as u8, Register::CanCtrl as u8, 0x55]
    );
}

#[tokio::test]
async fn modify_register_sends_bitmod() {
    let mut t = TestDevice::new();

    t.dev
        .modify_register(Register::CanCtrl, 0xF0, 0x0F)
        .await
        .unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::CanCtrl as u8,
            0xF0,
            0x0F
        ]
    );
}

#[tokio::test]
async fn set_bitrate_writes_all_three_registers() {
    let mut t = TestDevice::new();

    let cfg = BitrateConfig {
        cnf1: 0xAA,
        cnf2: 0xBB,
        cnf3: 0xCC,
    };

    t.dev.set_bitrate(cfg).await.unwrap();

    let writes = t.spi.writes();
    assert_eq!(writes.len(), 3);

    assert_eq!(
        writes[0],
        vec![Instruction::Write as u8, Register::Cnf1 as u8, 0xAA]
    );
    assert_eq!(
        writes[1],
        vec![Instruction::Write as u8, Register::Cnf2 as u8, 0xBB]
    );
    assert_eq!(
        writes[2],
        vec![Instruction::Write as u8, Register::Cnf3 as u8, 0xCC]
    );
}

#[tokio::test]
async fn set_bitrate_order_is_correct() {
    let mut t = TestDevice::new();

    let cfg = BitrateConfig {
        cnf1: 0x01,
        cnf2: 0x02,
        cnf3: 0x03,
    };

    t.dev.set_bitrate(cfg).await.unwrap();

    let regs: Vec<u8> = t.spi.writes().iter().map(|w| w[1]).collect();
    assert_eq!(
        regs,
        vec![
            Register::Cnf1 as u8,
            Register::Cnf2 as u8,
            Register::Cnf3 as u8
        ]
    );
}
