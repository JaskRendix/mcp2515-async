#![no_std]

pub mod can;
pub mod config;
pub mod error;
pub mod registers;

pub use config::{AcceptanceFilter, Bitrate, BitrateConfig, Clock, FilterMask};

use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;
use error::Error;
use registers::{Instruction, OperationMode, Register};

pub struct Mcp2515<SPI, INT = (), PinE = core::convert::Infallible> {
    spi: SPI,
    int_pin: INT,
    _pin_err: core::marker::PhantomData<PinE>,
}

impl<SPI, E> Mcp2515<SPI, (), core::convert::Infallible>
where
    SPI: SpiDevice<u8, Error = E>,
{
    /// Create a new MCP2515 driver instance without an interrupt pin (polling mode).
    pub const fn new(spi: SPI) -> Self {
        Self {
            spi,
            int_pin: (),
            _pin_err: core::marker::PhantomData,
        }
    }
}

impl<SPI, INT, E, PinE> Mcp2515<SPI, INT, PinE>
where
    SPI: SpiDevice<u8, Error = E>,
    INT: Wait<Error = PinE>,
{
    /// Create a new MCP2515 driver instance with an async interrupt pin.
    pub const fn new_with_interrupt(spi: SPI, int_pin: INT) -> Self {
        Self {
            spi,
            int_pin,
            _pin_err: core::marker::PhantomData,
        }
    }

    /// Asynchronously wait for the MCP2515 interrupt pin to go low (active-low signal).
    pub async fn wait_for_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.int_pin.wait_for_low().await.map_err(Error::Pin)?;
        Ok(())
    }

    /// Asynchronously wait for an interrupt signal, then read the incoming frame.
    pub async fn recv(&mut self) -> Result<can::CanFrame, Error<E, PinE>> {
        // Wait for the interrupt pin to go low (active-low message available signal)
        self.wait_for_interrupt().await?;
        // Read the frame from the RX buffer
        self.read().await
    }
}

impl<SPI, INT, E, PinE> Mcp2515<SPI, INT, PinE>
where
    SPI: SpiDevice<u8, Error = E>,
{
    pub async fn reset(&mut self) -> Result<(), Error<E, PinE>> {
        self.spi
            .write(&[Instruction::Reset as u8])
            .await
            .map_err(Error::Spi)
    }

    pub async fn set_mode(&mut self, mode: OperationMode) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0xE0, mode as u8)
            .await
    }

    pub async fn read_register(&mut self, reg: Register) -> Result<u8, Error<E, PinE>> {
        let buf = [Instruction::Read as u8, reg as u8, 0x00];
        let mut res = [0u8; 3];
        self.spi
            .transfer(&mut res, &buf)
            .await
            .map_err(Error::Spi)?;
        Ok(res[2])
    }

    pub async fn set_register(&mut self, reg: Register, val: u8) -> Result<(), Error<E, PinE>> {
        let buf = [Instruction::Write as u8, reg as u8, val];
        self.spi.write(&buf).await.map_err(Error::Spi)
    }

    pub async fn modify_register(
        &mut self,
        reg: Register,
        mask: u8,
        val: u8,
    ) -> Result<(), Error<E, PinE>> {
        let buf = [Instruction::BitMod as u8, reg as u8, mask, val];
        self.spi.write(&buf).await.map_err(Error::Spi)
    }

    pub async fn set_bitrate(&mut self, config: BitrateConfig) -> Result<(), Error<E, PinE>> {
        self.set_register(Register::Cnf1, config.cnf1).await?;
        self.set_register(Register::Cnf2, config.cnf2).await?;
        self.set_register(Register::Cnf3, config.cnf3).await?;
        Ok(())
    }

    /// Helper to write a 29-bit or 11-bit identifier into 4 consecutive registers (SIDH, SIDL, EID8, EID0)
    async fn write_id_registers(
        &mut self,
        start_reg: Register,
        is_extended: bool,
        id: u32,
    ) -> Result<(), Error<E, PinE>> {
        let (sidh, sidl, eid8, eid0) = if is_extended {
            let sidh = (id >> 21) as u8;
            let sidl = (((id >> 13) & 0xE0) as u8) | 0x08 | (((id >> 16) & 0x03) as u8);
            let eid8 = (id >> 8) as u8;
            let eid0 = id as u8;
            (sidh, sidl, eid8, eid0)
        } else {
            let sidh = (id >> 3) as u8;
            let sidl = ((id & 0x07) << 5) as u8;
            (sidh, sidl, 0, 0)
        };

        let base = start_reg as u8;
        // Safety: register addresses are sequential and valid u8 values mapped to Register enum
        self.set_register(unsafe { core::mem::transmute::<u8, Register>(base) }, sidh)
            .await?;
        self.set_register(
            unsafe { core::mem::transmute::<u8, Register>(base + 1) },
            sidl,
        )
        .await?;
        self.set_register(
            unsafe { core::mem::transmute::<u8, Register>(base + 2) },
            eid8,
        )
        .await?;
        self.set_register(
            unsafe { core::mem::transmute::<u8, Register>(base + 3) },
            eid0,
        )
        .await?;
        Ok(())
    }

    /// Configure an Acceptance Filter (RXF0 - RXF5)
    pub async fn set_filter(
        &mut self,
        filter: AcceptanceFilter,
        is_extended: bool,
        id: u32,
    ) -> Result<(), Error<E, PinE>> {
        let base_reg = match filter {
            AcceptanceFilter::Rxf0 => Register::RxF0SidH,
            AcceptanceFilter::Rxf1 => Register::RxF1SidH,
            AcceptanceFilter::Rxf2 => Register::RxF2SidH,
            AcceptanceFilter::Rxf3 => Register::RxF3SidH,
            AcceptanceFilter::Rxf4 => Register::RxF4SidH,
            AcceptanceFilter::Rxf5 => Register::RxF5SidH,
        };

        self.write_id_registers(base_reg, is_extended, id).await
    }

    /// Configure a Filter Mask (MASK0 or MASK1)
    pub async fn set_filter_mask(
        &mut self,
        mask: FilterMask,
        is_extended: bool,
        id: u32,
    ) -> Result<(), Error<E, PinE>> {
        let base_reg = match mask {
            FilterMask::Mask0 => Register::RxM0SidH,
            FilterMask::Mask1 => Register::RxM1SidH,
        };

        self.write_id_registers(base_reg, is_extended, id).await
    }

    /// Send a CAN frame using the first available transmit buffer (TXB0, TXB1, or TXB2).
    pub async fn send(&mut self, frame: &can::CanFrame) -> Result<(), Error<E, PinE>> {
        let txb0_ctrl = self.read_register(Register::TxB0Ctrl).await?;
        let (load_instr, rts_instr) = if (txb0_ctrl & 0x08) == 0 {
            (Instruction::LoadTx0, Instruction::RtsTx0)
        } else {
            let txb1_ctrl = self.read_register(Register::TxB1Ctrl).await?;
            if (txb1_ctrl & 0x08) == 0 {
                (Instruction::LoadTx1, Instruction::RtsTx1)
            } else {
                let txb2_ctrl = self.read_register(Register::TxB2Ctrl).await?;
                if (txb2_ctrl & 0x08) == 0 {
                    (Instruction::LoadTx2, Instruction::RtsTx2)
                } else {
                    return Err(Error::Spi(unsafe { core::mem::zeroed() }));
                }
            }
        };

        let is_ext = frame.is_extended();
        let (sidh, sidl, eid8, eid0, dlc_val) = if is_ext {
            let id = frame.extended_id();
            let sidh = (id >> 21) as u8;
            let sidl = (((id >> 13) & 0xE0) as u8) | 0x08 | (((id >> 16) & 0x03) as u8);
            let eid8 = (id >> 8) as u8;
            let eid0 = id as u8;
            (sidh, sidl, eid8, eid0, (frame.can_dlc & 0x0F) | 0x40)
        } else {
            let id = frame.standard_id() as u32;
            let sidh = (id >> 3) as u8;
            let sidl = ((id & 0x07) << 5) as u8;
            (sidh, sidl, 0, 0, frame.can_dlc & 0x0F)
        };

        let mut buf = [0u8; 14];
        buf[0] = load_instr as u8;
        buf[1] = sidh;
        buf[2] = sidl;
        buf[3] = eid8;
        buf[4] = eid0;
        buf[5] = dlc_val;

        let dlc_len = frame.can_dlc as usize;
        let mut i = 0;
        while i < dlc_len {
            buf[6 + i] = frame.data[i];
            i += 1;
        }

        self.spi
            .write(&buf[..6 + dlc_len])
            .await
            .map_err(Error::Spi)?;
        self.spi
            .write(&[rts_instr as u8])
            .await
            .map_err(Error::Spi)?;

        Ok(())
    }

    /// Read a received CAN frame from either RXB0 or RXB1 if available.
    pub async fn read(&mut self) -> Result<can::CanFrame, Error<E, PinE>> {
        let intf = self.read_register(Register::CanIntf).await?;

        let (read_instr, int_flag_mask) = if (intf & 0x01) != 0 {
            (Instruction::ReadRx0, 0xFE)
        } else if (intf & 0x02) != 0 {
            (Instruction::ReadRx1, 0xFD)
        } else {
            return Err(Error::Spi(unsafe { core::mem::zeroed() }));
        };

        let mut res = [0u8; 14];
        let cmd = [read_instr as u8];
        self.spi
            .transfer(&mut res, &cmd)
            .await
            .map_err(Error::Spi)?;

        let sidh = res[1];
        let sidl = res[2];
        let eid8 = res[3];
        let eid0 = res[4];
        let dlc_byte = res[5];

        let is_extended = (sidl & 0x08) != 0;
        let dlc = dlc_byte & 0x0F;

        let can_id = if is_extended {
            let raw_id = ((sidh as u32) << 21)
                | (((sidl & 0xE0) as u32) << 13)
                | (((sidl & 0x03) as u32) << 16)
                | ((eid8 as u32) << 8)
                | (eid0 as u32);
            can::CAN_EFF_FLAG | raw_id
        } else {
            ((sidh as u32) << 3) | ((sidl as u32) >> 5)
        };

        let mut data = [0u8; 8];
        let mut i = 0;
        while i < dlc as usize && i < 8 {
            data[i] = res[6 + i];
            i += 1;
        }

        self.modify_register(Register::CanIntf, !int_flag_mask, 0)
            .await?;

        can::CanFrame::new(can_id, &data[..dlc as usize])
            .ok_or_else(|| Error::Spi(unsafe { core::mem::zeroed() }))
    }

    /// Read the Transmit Error Counter (TEC).
    pub async fn read_tec(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::Tec).await
    }

    /// Read the Receive Error Counter (REC).
    pub async fn read_rec(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::Rec).await
    }

    /// Read the Error Flag Register (EFLG).
    pub async fn read_eflg(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::Eflg).await
    }

    /// Check if the controller has entered the Bus-Off state (TXBO bit in EFLG).
    pub async fn is_bus_off(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x20) != 0)
    }
}
