mod common;

use common::harness::TestDevice;
use mcp2515_async::BitrateConfig;
use mcp2515_async::can::{CAN_RTR_FLAG, CanFrame};
use mcp2515_async::registers::{Instruction, OperationMode, Register};
use mcp2515_async::{AcceptanceFilter, FilterMask, TxPriority};

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

#[tokio::test]
async fn rx0_pending_detects_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x01]);

    assert!(t.dev.rx0_pending().await.unwrap());
}

#[tokio::test]
async fn rx1_pending_detects_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x02]);

    assert!(t.dev.rx1_pending().await.unwrap());
}

#[tokio::test]
async fn error_interrupt_pending_detects_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x20]);

    assert!(t.dev.error_interrupt_pending().await.unwrap());
}

#[tokio::test]
async fn is_error_warning_detects_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x01]);

    assert!(t.dev.is_error_warning().await.unwrap());
}

#[tokio::test]
async fn is_rx_warning_detects_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x02]);

    assert!(t.dev.is_rx_warning().await.unwrap());
}

#[tokio::test]
async fn is_tx_warning_detects_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x04]);

    assert!(t.dev.is_tx_warning().await.unwrap());
}

#[tokio::test]
async fn rx0_overflow_detects_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x40]);

    assert!(t.dev.rx0_overflowed().await.unwrap());
}

#[tokio::test]
async fn rx1_overflow_detects_flag() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x80]);

    assert!(t.dev.rx1_overflowed().await.unwrap());
}

#[tokio::test]
async fn read_status_returns_status_byte() {
    let mut t = TestDevice::with_transfer_output(&[0, 0x03]);

    let status = t.dev.read_status().await.unwrap();

    assert_eq!(status, 0x03);
}

#[tokio::test]
async fn rx_status_returns_status_byte() {
    let mut t = TestDevice::with_transfer_output(&[0, 0xAA]);

    let status = t.dev.rx_status().await.unwrap();

    assert_eq!(status, 0xAA);
}

#[tokio::test]
async fn enable_one_shot_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.enable_one_shot().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::CanCtrl as u8,
            0x08,
            0x08,
        ]
    );
}

#[tokio::test]
async fn enable_rollover_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.enable_rollover().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::RxB0Ctrl as u8,
            0x04,
            0x04,
        ]
    );
}

#[tokio::test]
async fn read_registers_reads_multiple_bytes() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x11, 0x22, 0x33]);

    let mut data = [0u8; 3];
    t.dev
        .read_registers(Register::CanCtrl, &mut data)
        .await
        .unwrap();

    assert_eq!(data, [0x11, 0x22, 0x33]);
}

#[tokio::test]
async fn write_registers_writes_multiple_bytes() {
    let mut t = TestDevice::new();

    t.dev
        .write_registers(Register::CanCtrl, &[0x11, 0x22, 0x33])
        .await
        .unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::Write as u8,
            Register::CanCtrl as u8,
            0x11,
            0x22,
            0x33,
        ]
    );
}

#[tokio::test]
async fn abort_all_transmissions_sets_and_clears_abat() {
    let mut t = TestDevice::new();

    t.dev.abort_all_transmissions().await.unwrap();

    let writes = t.spi.writes();

    assert_eq!(
        writes[0],
        vec![
            Instruction::BitMod as u8,
            Register::CanCtrl as u8,
            0x10,
            0x10,
        ]
    );

    assert_eq!(
        writes[1],
        vec![
            Instruction::BitMod as u8,
            Register::CanCtrl as u8,
            0x10,
            0x00,
        ]
    );
}

#[tokio::test]
async fn enable_wakeup_filter_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.enable_wakeup_filter().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::Cnf3 as u8,
            0x40,
            0x40,
        ]
    );
}

#[tokio::test]
async fn disable_wakeup_filter_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.disable_wakeup_filter().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::Cnf3 as u8,
            0x40,
            0x00,
        ]
    );
}

#[tokio::test]
async fn enable_start_of_frame_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.enable_start_of_frame().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::Cnf3 as u8,
            0x80,
            0x80,
        ]
    );
}

#[tokio::test]
async fn disable_start_of_frame_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.disable_start_of_frame().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::Cnf3 as u8,
            0x80,
            0x00,
        ]
    );
}

#[tokio::test]
async fn clear_rx0_overflow_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.clear_rx0_overflow().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::Eflg as u8,
            0x40,
            0x00,
        ]
    );
}

#[tokio::test]
async fn clear_overflows_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.clear_overflows().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::Eflg as u8,
            0xC0,
            0x00,
        ]
    );
}

#[tokio::test]
async fn enable_clkout_writes_correct_mask() {
    let mut t = TestDevice::new();

    t.dev.enable_clkout().await.unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::CanCtrl as u8,
            0x04,
            0x04,
        ]
    );
}

#[tokio::test]
async fn interrupt_code_extracts_icod_bits() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0b0000_1010]);

    let code = t.dev.interrupt_code().await.unwrap();

    assert_eq!(code, 0b101);
}

#[tokio::test]
async fn tx0_busy_detects_txreq() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x08]);

    assert!(t.dev.tx0_busy().await.unwrap());
}

#[tokio::test]
async fn tx0_error_detects_txerr() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x10]);

    assert!(t.dev.tx0_error().await.unwrap());
}

#[tokio::test]
async fn tx0_lost_arbitration_detects_mloa() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x20]);

    assert!(t.dev.tx0_lost_arbitration().await.unwrap());
}

#[tokio::test]
async fn tx0_aborted_detects_abtf() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x40]);

    assert!(t.dev.tx0_aborted().await.unwrap());
}

#[tokio::test]
async fn read_registers_rejects_large_buffer() {
    let mut t = TestDevice::new();

    let mut buf = [0u8; 63];

    assert!(t.dev.read_registers(Register::CanCtrl, &mut buf).await.is_err());
}

#[tokio::test]
async fn write_registers_rejects_large_buffer() {
    let mut t = TestDevice::new();

    let buf = [0u8; 63];

    assert!(t.dev.write_registers(Register::CanCtrl, &buf).await.is_err());
}

#[tokio::test]
async fn one_shot_enabled_detects_bit() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x08]);

    assert!(t.dev.is_one_shot_enabled().await.unwrap());
}

#[tokio::test]
async fn rollover_enabled_detects_bit() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x04]);

    assert!(t.dev.rollover_enabled().await.unwrap());
}

#[tokio::test]
async fn clkout_enabled_detects_bit() {
    let mut t = TestDevice::with_transfer_output(&[0, 0, 0x04]);

    assert!(t.dev.clkout_enabled().await.unwrap());
}

#[tokio::test]
async fn set_tx0_priority_lowest() {
    let mut t = TestDevice::new();

    t.dev
        .set_tx0_priority(TxPriority::Lowest)
        .await
        .unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::TxB0Ctrl as u8,
            0x03,
            0x00,
        ]
    );
}

#[tokio::test]
async fn set_tx0_priority_highest() {
    let mut t = TestDevice::new();

    t.dev
        .set_tx0_priority(TxPriority::Highest)
        .await
        .unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::TxB0Ctrl as u8,
            0x03,
            0x03,
        ]
    );
}

#[tokio::test]
async fn set_tx1_priority_high() {
    let mut t = TestDevice::new();

    t.dev
        .set_tx1_priority(TxPriority::High)
        .await
        .unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::TxB1Ctrl as u8,
            0x03,
            0x02,
        ]
    );
}

#[tokio::test]
async fn set_tx2_priority_low() {
    let mut t = TestDevice::new();

    t.dev
        .set_tx2_priority(TxPriority::Low)
        .await
        .unwrap();

    assert_eq!(
        t.spi.writes()[0],
        vec![
            Instruction::BitMod as u8,
            Register::TxB2Ctrl as u8,
            0x03,
            0x01,
        ]
    );
}
