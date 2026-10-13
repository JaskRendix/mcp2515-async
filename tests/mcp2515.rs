mod common;

use common::harness::TestDevice;
use mcp2515_async::BitrateConfig;
use mcp2515_async::can::{CAN_RTR_FLAG, CanFrame};
use mcp2515_async::registers::{Instruction, OperationMode, Register};
use mcp2515_async::{AcceptanceFilter, FilterMask};

#[tokio::test]
async fn reset_sends_correct_instruction() {
    let mut t = TestDevice::new();

    t.dev.reset().await.unwrap();

    let writes = t.writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0], vec![Instruction::Reset as u8]);
}

#[tokio::test]
async fn set_mode_waits_for_requested_mode() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, OperationMode::Normal as u8]);

    t.dev.set_mode(OperationMode::Normal).await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::CanCtrl as u8,
            0xE0,
            OperationMode::Normal as u8,
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

#[tokio::test]
async fn set_mode_times_out_if_mode_never_changes() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0]);

    let err = t.dev.set_mode(OperationMode::Loopback).await.unwrap_err();

    assert_eq!(err, mcp2515_async::error::Error::ModeChangeTimeout);
}

#[tokio::test]
async fn send_sets_rtr_bit_in_dlc() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0]);

    let frame = CanFrame::new(CAN_RTR_FLAG | 0x123, &[]).unwrap();

    t.dev.send(&frame).await.unwrap();

    let writes = t.spi.writes();

    assert_eq!(writes.len(), 2);

    assert_eq!(writes[0][0], Instruction::LoadTx0 as u8);

    assert_eq!(writes[0][5], 0x40);
}

#[tokio::test]
async fn set_filter_writes_standard_id_registers() {
    let mut t = TestDevice::new();

    t.dev
        .set_filter(AcceptanceFilter::Rxf0, false, 0x123)
        .await
        .unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::Write as u8,
            Register::RxF0SidH as u8,
            0x24,
            0x60,
            0x00,
            0x00,
        ]
    );
}

#[tokio::test]
async fn set_filter_mask_writes_standard_id_registers() {
    let mut t = TestDevice::new();

    t.dev
        .set_filter_mask(FilterMask::Mask0, false, 0x123)
        .await
        .unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::Write as u8,
            Register::RxM0SidH as u8,
            0x24,
            0x60,
            0x00,
            0x00,
        ]
    );
}

#[tokio::test]
async fn read_tec_returns_register_value() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x55]);

    let tec = t.dev.read_tec().await.unwrap();

    assert_eq!(tec, 0x55);
}

#[tokio::test]
async fn read_rec_returns_register_value() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x66]);

    let rec = t.dev.read_rec().await.unwrap();

    assert_eq!(rec, 0x66);
}

#[tokio::test]
async fn read_eflg_returns_register_value() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x77]);

    let eflg = t.dev.read_eflg().await.unwrap();

    assert_eq!(eflg, 0x77);
}

#[tokio::test]
async fn is_bus_off_detects_bus_off_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x20]);

    assert!(t.dev.is_bus_off().await.unwrap());
}

#[tokio::test]
async fn is_bus_off_returns_false_when_flag_not_set() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x00]);

    assert!(!t.dev.is_bus_off().await.unwrap());
}
