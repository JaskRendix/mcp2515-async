pub type CanId = u32;

pub const CAN_EFF_FLAG: u32 = 0x8000_0000;
pub const CAN_RTR_FLAG: u32 = 0x4000_0000;
pub const CAN_ERR_FLAG: u32 = 0x2000_0000;

pub const CAN_SFF_MASK: u32 = 0x0000_07FF;
pub const CAN_EFF_MASK: u32 = 0x1FFFFFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanFrame {
    pub can_id: CanId,
    pub can_dlc: u8,
    pub data: [u8; 8],
}

impl CanFrame {
    pub const fn new(can_id: CanId, data: &[u8]) -> Option<Self> {
        if data.len() > 8 {
            return None;
        }
        let mut d = [0u8; 8];
        let mut i = 0;
        while i < data.len() {
            d[i] = data[i];
            i += 1;
        }
        Some(Self {
            can_id,
            can_dlc: data.len() as u8,
            data: d,
        })
    }

    pub const fn is_extended(&self) -> bool {
        (self.can_id & CAN_EFF_FLAG) != 0
    }

    pub const fn is_rtr(&self) -> bool {
        (self.can_id & CAN_ERR_FLAG) == 0 && (self.can_id & CAN_RTR_FLAG) != 0
    }

    pub const fn standard_id(&self) -> u16 {
        (self.can_id & CAN_SFF_MASK) as u16
    }

    pub const fn extended_id(&self) -> u32 {
        self.can_id & CAN_EFF_MASK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_valid_lengths() {
        let f = CanFrame::new(0x123, &[1, 2, 3]).unwrap();
        assert_eq!(f.can_dlc, 3);
        assert_eq!(f.data[..3], [1, 2, 3]);
        assert_eq!(f.data[3], 0);
    }

    #[test]
    fn new_rejects_too_long() {
        let data = [0u8; 9];
        assert!(CanFrame::new(0x123, &data).is_none());
    }

    #[test]
    fn new_zero_length_is_valid() {
        let f = CanFrame::new(0x123, &[]).unwrap();
        assert_eq!(f.can_dlc, 0);
        assert_eq!(f.data, [0; 8]);
    }

    #[test]
    fn new_exact_eight_bytes() {
        let d = [1, 2, 3, 4, 5, 6, 7, 8];
        let f = CanFrame::new(0x123, &d).unwrap();
        assert_eq!(f.can_dlc, 8);
        assert_eq!(f.data, d);
    }

    #[test]
    fn detects_standard_frame() {
        let f = CanFrame::new(0x123, &[0]).unwrap();
        assert!(!f.is_extended());
        assert_eq!(f.standard_id(), 0x123);
    }

    #[test]
    fn detects_extended_frame() {
        let id = CAN_EFF_FLAG | 0x1ABCDE;
        let f = CanFrame::new(id, &[0]).unwrap();
        assert!(f.is_extended());
        assert_eq!(f.extended_id(), 0x1ABCDE);
    }

    #[test]
    fn extended_id_masks_correctly() {
        let id = CAN_EFF_FLAG | CAN_EFF_MASK; // max extended ID
        let f = CanFrame::new(id, &[0]).unwrap();
        assert_eq!(f.extended_id(), CAN_EFF_MASK);
    }

    #[test]
    fn detects_rtr_frame() {
        let id = CAN_RTR_FLAG | 0x321;
        let f = CanFrame::new(id, &[]).unwrap();
        assert!(f.is_rtr());
    }

    #[test]
    fn rtr_is_false_for_error_frames() {
        let id = CAN_ERR_FLAG | CAN_RTR_FLAG | 0x321;
        let f = CanFrame::new(id, &[]).unwrap();
        assert!(!f.is_rtr());
    }

    #[test]
    fn rtr_is_false_when_flag_not_set() {
        let f = CanFrame::new(0x123, &[]).unwrap();
        assert!(!f.is_rtr());
    }

    #[test]
    fn standard_id_masks_correctly() {
        let id = 0xFFFF_FFFF; // high bits ignored
        let f = CanFrame::new(id, &[0]).unwrap();
        assert_eq!(f.standard_id(), CAN_SFF_MASK as u16);
    }

    #[test]
    fn extended_id_masks_correctly_again() {
        let id = CAN_EFF_FLAG | 0xDEADBEEF;
        let f = CanFrame::new(id, &[0]).unwrap();
        assert_eq!(f.extended_id(), 0x1EADBEEF);
    }
}
