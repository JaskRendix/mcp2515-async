use mcp2515_async::can::{CAN_EFF_FLAG, CAN_ERR_FLAG, CAN_RTR_FLAG, CAN_SFF_MASK, CanFrame};

#[test]
fn construct_standard_frame() {
    let f = CanFrame::new(0x123, &[1, 2, 3]).unwrap();
    assert_eq!(f.can_id, 0x123);
    assert_eq!(f.can_dlc, 3);
    assert_eq!(f.data[..3], [1, 2, 3]);
}

#[test]
fn construct_extended_frame() {
    let id = CAN_EFF_FLAG | 0x1ABCDE;
    let f = CanFrame::new(id, &[0]).unwrap();
    assert!(f.is_extended());
    assert_eq!(f.extended_id(), 0x1ABCDE);
}

#[test]
fn rtr_flag_detected() {
    let id = CAN_RTR_FLAG | 0x321;
    let f = CanFrame::new(id, &[]).unwrap();
    assert!(f.is_rtr());
}

#[test]
fn rtr_not_set_for_error_frames() {
    let id = CAN_ERR_FLAG | CAN_RTR_FLAG | 0x321;
    let f = CanFrame::new(id, &[]).unwrap();
    assert!(!f.is_rtr());
}

#[test]
fn standard_id_masking() {
    let f = CanFrame::new(0xFFFF_FFFF, &[0]).unwrap();
    assert_eq!(f.standard_id(), CAN_SFF_MASK as u16);
}

#[test]
fn extended_id_masking() {
    let id = CAN_EFF_FLAG | 0xDEADBEEF;
    let f = CanFrame::new(id, &[0]).unwrap();
    assert_eq!(f.extended_id(), 0x1EADBEEF);
}

#[test]
fn reject_dlc_greater_than_8() {
    let data = [0u8; 9];
    assert!(CanFrame::new(0x123, &data).is_err());
}

#[test]
fn reject_invalid_standard_id() {
    assert!(CanFrame::new(0xFFFF, &[1]).is_err());
}

#[test]
fn accept_max_extended_id() {
    let id = CAN_EFF_FLAG | 0x1FFF_FFFF;
    let f = CanFrame::new(id, &[1]).unwrap();

    assert!(f.is_extended());
    assert_eq!(f.extended_id(), 0x1FFF_FFFF);
}

#[test]
fn error_flag_detected() {
    let id = CAN_ERR_FLAG | 0x123;
    let f = CanFrame::new(id, &[]).unwrap();

    assert!(f.is_error());
}

#[test]
fn accept_max_standard_id() {
    let f = CanFrame::new(0x7FF, &[0]).unwrap();
    assert_eq!(f.standard_id(), 0x7FF);
}

#[test]
fn id_returns_extended_id() {
    let id = CAN_EFF_FLAG | 0x123456;
    let f = CanFrame::new(id, &[0]).unwrap();

    assert_eq!(f.id(), 0x123456);
}

#[test]
fn id_returns_standard_id() {
    let f = CanFrame::new(0x456, &[0]).unwrap();

    assert_eq!(f.id(), 0x456);
}

#[test]
fn reject_standard_id_above_max() {
    assert!(CanFrame::new(0x800, &[0]).is_err());
}

#[test]
fn reject_extended_id_above_max() {
    let id = CAN_EFF_FLAG | 0x2000_0000;
    assert!(CanFrame::new(id, &[0]).is_err());
}

#[test]
fn accept_empty_frame() {
    let f = CanFrame::new(0x123, &[]).unwrap();

    assert_eq!(f.can_dlc, 0);
}

#[test]
fn accept_full_dlc_frame() {
    let data = [1, 2, 3, 4, 5, 6, 7, 8];

    let f = CanFrame::new(0x123, &data).unwrap();

    assert_eq!(f.can_dlc, 8);
    assert_eq!(f.data, data);
}

#[test]
fn standard_frame_is_not_extended() {
    let f = CanFrame::new(0x123, &[0]).unwrap();

    assert!(!f.is_extended());
}

#[test]
fn standard_frame_is_not_error() {
    let f = CanFrame::new(0x123, &[0]).unwrap();

    assert!(!f.is_error());
}

#[test]
fn standard_frame_is_not_rtr() {
    let f = CanFrame::new(0x123, &[0]).unwrap();

    assert!(!f.is_rtr());
}

#[test]
fn extended_frame_is_not_rtr_by_default() {
    let f = CanFrame::new(CAN_EFF_FLAG | 0x123456, &[0]).unwrap();

    assert!(!f.is_rtr());
}

#[test]
fn extended_frame_is_not_error_by_default() {
    let f = CanFrame::new(CAN_EFF_FLAG | 0x123456, &[0]).unwrap();

    assert!(!f.is_error());
}

#[test]
fn error_frame_is_not_extended() {
    let f = CanFrame::new(CAN_ERR_FLAG | 0x123, &[]).unwrap();

    assert!(!f.is_extended());
}

#[test]
fn data_is_copied_into_frame() {
    let mut payload = [1, 2, 3];

    let frame = CanFrame::new(0x123, &payload).unwrap();

    payload[0] = 99;

    assert_eq!(payload[0], 99);
    assert_eq!(frame.data[0], 1);
}

#[test]
fn all_identifier_flags_can_coexist() {
    let id = CAN_EFF_FLAG | CAN_RTR_FLAG | 0x123456;

    let frame = CanFrame::new(id, &[]).unwrap();

    assert!(frame.is_extended());
    assert!(frame.is_rtr());
    assert_eq!(frame.extended_id(), 0x123456);
}

#[test]
fn extended_error_frame_is_not_rtr() {
    let id = CAN_EFF_FLAG | CAN_ERR_FLAG | CAN_RTR_FLAG | 0x123456;

    let frame = CanFrame::new(id, &[]).unwrap();

    assert!(frame.is_error());
    assert!(!frame.is_rtr());
}

#[test]
fn accept_minimum_standard_id() {
    let frame = CanFrame::new(0x0, &[0]).unwrap();

    assert_eq!(frame.standard_id(), 0);
}

#[test]
fn accept_minimum_extended_id() {
    let frame = CanFrame::new(CAN_EFF_FLAG, &[0]).unwrap();

    assert!(frame.is_extended());
    assert_eq!(frame.extended_id(), 0);
}
