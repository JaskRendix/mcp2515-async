pub type CanId = u32;

pub const CAN_EFF_FLAG: u32 = 0x8000_0000;
pub const CAN_RTR_FLAG: u32 = 0x4000_0000;
pub const CAN_ERR_FLAG: u32 = 0x2000_0000;

pub const CAN_FLAG_MASK: u32 = CAN_EFF_FLAG | CAN_RTR_FLAG | CAN_ERR_FLAG;

pub const CAN_SFF_MASK: u32 = 0x0000_07FF;
pub const CAN_EFF_MASK: u32 = 0x1FFF_FFFF;

pub const CAN_MAX_DLEN: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    InvalidDlc,
    InvalidStandardId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanFrame {
    pub can_id: CanId,
    pub can_dlc: u8,
    pub data: [u8; CAN_MAX_DLEN],
}

impl CanFrame {
    pub const fn new(can_id: CanId, data: &[u8]) -> Result<Self, FrameError> {
        if data.len() > CAN_MAX_DLEN {
            return Err(FrameError::InvalidDlc);
        }

        let is_extended = (can_id & CAN_EFF_FLAG) != 0;

        if !is_extended && (can_id & !(CAN_SFF_MASK | CAN_FLAG_MASK)) != 0 {
            return Err(FrameError::InvalidStandardId);
        }

        let mut d = [0u8; CAN_MAX_DLEN];
        let mut i = 0;

        while i < data.len() {
            d[i] = data[i];
            i += 1;
        }

        Ok(Self {
            can_id,
            can_dlc: data.len() as u8,
            data: d,
        })
    }

    pub const fn is_extended(&self) -> bool {
        (self.can_id & CAN_EFF_FLAG) != 0
    }

    pub const fn is_rtr(&self) -> bool {
        !self.is_error() && (self.can_id & CAN_RTR_FLAG) != 0
    }

    pub const fn is_error(&self) -> bool {
        (self.can_id & CAN_ERR_FLAG) != 0
    }

    pub const fn standard_id(&self) -> u16 {
        (self.can_id & CAN_SFF_MASK) as u16
    }

    pub const fn extended_id(&self) -> u32 {
        self.can_id & CAN_EFF_MASK
    }

    pub const fn id(&self) -> u32 {
        if self.is_extended() {
            self.extended_id()
        } else {
            self.standard_id() as u32
        }
    }
}
