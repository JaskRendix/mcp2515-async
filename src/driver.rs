use crate::can;
use crate::config::{AcceptanceFilter, BitrateConfig, FilterMask, TxPriority};
use crate::error::Error;
use crate::registers::{Instruction, OperationMode, Register};

use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

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
        loop {
            self.wait_for_interrupt().await?;

            match self.read().await {
                Err(Error::NoMessageAvailable) => continue,
                result => return result,
            }
        }
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
            .await?;

        for _ in 0..100 {
            let stat = self.read_register(Register::CanStat).await?;

            if (stat & 0xE0) == (mode as u8) {
                return Ok(());
            }
        }

        Err(Error::ModeChangeTimeout)
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

        let buf = [
            Instruction::Write as u8,
            start_reg as u8,
            sidh,
            sidl,
            eid8,
            eid0,
        ];
        self.spi.write(&buf).await.map_err(Error::Spi)
    }

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

        if !is_extended && id > 0x7FF {
            return Err(Error::InvalidFrame);
        }

        if is_extended && id > 0x1FFF_FFFF {
            return Err(Error::InvalidFrame);
        }

        self.write_id_registers(base_reg, is_extended, id).await
    }

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

        if !is_extended && id > 0x7FF {
            return Err(Error::InvalidFrame);
        }

        if is_extended && id > 0x1FFF_FFFF {
            return Err(Error::InvalidFrame);
        }

        self.write_id_registers(base_reg, is_extended, id).await
    }

    pub async fn send(&mut self, frame: &can::CanFrame) -> Result<(), Error<E, PinE>> {
        if frame.can_dlc > 8 {
            return Err(Error::InvalidDlc);
        }

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
                    return Err(Error::NoTxBufferAvailable);
                }
            }
        };

        let mut dlc_val = frame.can_dlc & 0x0F;

        if frame.is_rtr() {
            dlc_val |= 0x40;
        }

        let is_ext = frame.is_extended();
        let (sidh, sidl, eid8, eid0) = if is_ext {
            let id = frame.extended_id();
            let sidh = (id >> 21) as u8;
            let sidl = (((id >> 13) & 0xE0) as u8) | 0x08 | (((id >> 16) & 0x03) as u8);
            let eid8 = (id >> 8) as u8;
            let eid0 = id as u8;
            (sidh, sidl, eid8, eid0)
        } else {
            let id = frame.standard_id() as u32;
            let sidh = (id >> 3) as u8;
            let sidl = ((id & 0x07) << 5) as u8;
            (sidh, sidl, 0, 0)
        };

        let dlc_len = frame.can_dlc as usize;
        let mut buf = [0u8; 14];
        buf[0] = load_instr as u8;
        buf[1] = sidh;
        buf[2] = sidl;
        buf[3] = eid8;
        buf[4] = eid0;
        buf[5] = dlc_val;

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

    pub async fn read(&mut self) -> Result<can::CanFrame, Error<E, PinE>> {
        let intf = self.read_register(Register::CanIntf).await?;

        let (read_instr, clear_mask) = if (intf & 0x01) != 0 {
            (Instruction::ReadRx0, 0x01)
        } else if (intf & 0x02) != 0 {
            (Instruction::ReadRx1, 0x02)
        } else {
            return Err(Error::NoMessageAvailable);
        };

        let mut res = [0u8; 14];
        let mut tx = [0u8; 14];
        tx[0] = read_instr as u8;
        self.spi.transfer(&mut res, &tx).await.map_err(Error::Spi)?;

        let sidh = res[1];
        let sidl = res[2];
        let eid8 = res[3];
        let eid0 = res[4];
        let dlc_byte = res[5];

        let is_extended = (sidl & 0x08) != 0;

        let is_rtr = if is_extended {
            (dlc_byte & 0x40) != 0
        } else {
            (sidl & 0x10) != 0
        };

        let dlc = dlc_byte & 0x0F;

        if dlc > 8 {
            return Err(Error::InvalidDlc);
        }

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

        let can_id = if is_rtr {
            can_id | can::CAN_RTR_FLAG
        } else {
            can_id
        };

        let mut data = [0u8; 8];
        let mut i = 0;
        while i < dlc as usize && i < 8 {
            data[i] = res[6 + i];
            i += 1;
        }

        let frame =
            can::CanFrame::new(can_id, &data[..dlc as usize]).map_err(|_| Error::InvalidFrame)?;

        self.modify_register(Register::CanIntf, clear_mask, 0)
            .await?;

        Ok(frame)
    }

    async fn set_tx_priority(
        &mut self,
        reg: Register,
        priority: TxPriority,
    ) -> Result<(), Error<E, PinE>> {
        self.modify_register(reg, 0x03, priority as u8).await
    }

    pub async fn set_tx0_priority(&mut self, priority: TxPriority) -> Result<(), Error<E, PinE>> {
        self.set_tx_priority(Register::TxB0Ctrl, priority).await
    }

    pub async fn set_tx1_priority(&mut self, priority: TxPriority) -> Result<(), Error<E, PinE>> {
        self.set_tx_priority(Register::TxB1Ctrl, priority).await
    }

    pub async fn set_tx2_priority(&mut self, priority: TxPriority) -> Result<(), Error<E, PinE>> {
        self.set_tx_priority(Register::TxB2Ctrl, priority).await
    }

    pub async fn read_tec(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::Tec).await
    }

    pub async fn read_rec(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::Rec).await
    }

    pub async fn read_eflg(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::Eflg).await
    }

    pub async fn is_bus_off(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x20) != 0)
    }

    pub async fn is_error_warning(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x01) != 0)
    }

    pub async fn is_rx_warning(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x02) != 0)
    }

    pub async fn is_tx_warning(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x04) != 0)
    }

    pub async fn is_rx_error_passive(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x08) != 0)
    }

    pub async fn is_tx_error_passive(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x10) != 0)
    }

    pub async fn read_interrupt_flags(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::CanIntf).await
    }

    pub async fn read_interrupt_enable(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::CanInte).await
    }

    pub async fn rx0_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & 0x01) != 0)
    }

    pub async fn rx1_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & 0x02) != 0)
    }

    pub async fn error_interrupt_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & 0x20) != 0)
    }

    pub async fn enable_rx_interrupts(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, 0x03, 0x03).await
    }

    pub async fn disable_rx_interrupts(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, 0x03, 0x00).await
    }

    pub async fn clear_rx0_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanIntf, 0x01, 0).await
    }

    pub async fn clear_rx1_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanIntf, 0x02, 0).await
    }

    pub async fn clear_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanIntf, 0x20, 0).await
    }

    pub async fn clear_interrupt_flags(&mut self) -> Result<(), Error<E, PinE>> {
        self.set_register(Register::CanIntf, 0x00).await
    }

    pub async fn enable_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, 0x20, 0x20).await
    }

    pub async fn disable_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, 0x20, 0x00).await
    }

    pub async fn enable_wakeup_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, 0x40, 0x40).await
    }

    pub async fn disable_wakeup_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, 0x40, 0x00).await
    }

    pub async fn enable_tx_interrupts(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, 0x1C, 0x1C).await
    }

    pub async fn disable_tx_interrupts(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, 0x1C, 0x00).await
    }

    pub async fn rx0_overflowed(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x40) != 0)
    }

    pub async fn rx1_overflowed(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & 0x80) != 0)
    }

    pub async fn clear_rx0_overflow(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Eflg, 0x40, 0).await
    }

    pub async fn clear_rx1_overflow(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Eflg, 0x80, 0).await
    }

    pub async fn clear_overflows(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Eflg, 0xC0, 0).await
    }

    pub async fn tx0_busy(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB0Ctrl).await?;
        Ok((ctrl & 0x08) != 0)
    }

    pub async fn tx1_busy(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB1Ctrl).await?;
        Ok((ctrl & 0x08) != 0)
    }

    pub async fn tx2_busy(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB2Ctrl).await?;
        Ok((ctrl & 0x08) != 0)
    }

    pub async fn tx0_error(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB0Ctrl).await?;
        Ok((ctrl & 0x10) != 0)
    }

    pub async fn tx1_error(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB1Ctrl).await?;
        Ok((ctrl & 0x10) != 0)
    }

    pub async fn tx2_error(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB2Ctrl).await?;
        Ok((ctrl & 0x10) != 0)
    }

    pub async fn tx0_lost_arbitration(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB0Ctrl).await?;
        Ok((ctrl & 0x20) != 0)
    }

    pub async fn tx1_lost_arbitration(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB1Ctrl).await?;
        Ok((ctrl & 0x20) != 0)
    }

    pub async fn tx2_lost_arbitration(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB2Ctrl).await?;
        Ok((ctrl & 0x20) != 0)
    }

    pub async fn tx0_aborted(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB0Ctrl).await?;
        Ok((ctrl & 0x40) != 0)
    }

    pub async fn tx1_aborted(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB1Ctrl).await?;
        Ok((ctrl & 0x40) != 0)
    }

    pub async fn tx2_aborted(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB2Ctrl).await?;
        Ok((ctrl & 0x40) != 0)
    }

    pub async fn abort_all_transmissions(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x10, 0x10).await?;
        self.modify_register(Register::CanCtrl, 0x10, 0x00).await
    }

    pub async fn enable_one_shot(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x08, 0x08).await
    }

    pub async fn disable_one_shot(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x08, 0x00).await
    }

    pub async fn is_one_shot_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::CanCtrl).await?;
        Ok((ctrl & 0x08) != 0)
    }

    pub async fn enable_clkout(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x04, 0x04).await
    }

    pub async fn disable_clkout(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x04, 0x00).await
    }

    pub async fn set_clkout_div1(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x03, 0x00).await
    }

    pub async fn set_clkout_div2(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x03, 0x01).await
    }

    pub async fn set_clkout_div4(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x03, 0x02).await
    }

    pub async fn set_clkout_div8(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, 0x03, 0x03).await
    }

    pub async fn enable_rollover(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::RxB0Ctrl, 0x04, 0x04).await
    }

    pub async fn disable_rollover(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::RxB0Ctrl, 0x04, 0x00).await
    }

    pub async fn read_status(&mut self) -> Result<u8, Error<E, PinE>> {
        let tx = [0xA0, 0x00];
        let mut rx = [0u8; 2];

        self.spi.transfer(&mut rx, &tx).await.map_err(Error::Spi)?;

        Ok(rx[1])
    }

    pub async fn message_available(&mut self) -> Result<bool, Error<E, PinE>> {
        let status = self.read_status().await?;
        Ok((status & 0x03) != 0)
    }

    pub async fn rx_status(&mut self) -> Result<u8, Error<E, PinE>> {
        let tx = [0xB0, 0x00];
        let mut rx = [0u8; 2];

        self.spi.transfer(&mut rx, &tx).await.map_err(Error::Spi)?;

        Ok(rx[1])
    }

    pub async fn read_registers(
        &mut self,
        start: Register,
        data: &mut [u8],
    ) -> Result<(), Error<E, PinE>> {
        if data.len() > 62 {
            return Err(Error::InvalidFrame);
        }

        let mut tx = [0u8; 64];
        let mut rx = [0u8; 64];

        tx[0] = Instruction::Read as u8;
        tx[1] = start as u8;

        self.spi
            .transfer(&mut rx[..data.len() + 2], &tx[..data.len() + 2])
            .await
            .map_err(Error::Spi)?;

        data.copy_from_slice(&rx[2..data.len() + 2]);

        Ok(())
    }

    pub async fn write_registers(
        &mut self,
        start: Register,
        data: &[u8],
    ) -> Result<(), Error<E, PinE>> {
        if data.len() > 62 {
            return Err(Error::InvalidFrame);
        }

        let mut buf = [0u8; 64];

        buf[0] = Instruction::Write as u8;
        buf[1] = start as u8;

        let len = data.len();
        buf[2..2 + len].copy_from_slice(data);

        self.spi.write(&buf[..len + 2]).await.map_err(Error::Spi)
    }

    pub async fn rollover_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::RxB0Ctrl).await?;
        Ok((ctrl & 0x04) != 0)
    }

    pub async fn clkout_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::CanCtrl).await?;
        Ok((ctrl & 0x04) != 0)
    }

    pub async fn interrupt_code(&mut self) -> Result<u8, Error<E, PinE>> {
        let stat = self.read_register(Register::CanStat).await?;
        Ok((stat >> 1) & 0x07)
    }

    pub async fn enable_wakeup_filter(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Cnf3, 0x40, 0x40).await
    }

    pub async fn disable_wakeup_filter(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Cnf3, 0x40, 0x00).await
    }

    pub async fn enable_start_of_frame(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Cnf3, 0x80, 0x80).await
    }

    pub async fn disable_start_of_frame(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Cnf3, 0x80, 0x00).await
    }

    pub async fn wakeup_filter_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let cnf3 = self.read_register(Register::Cnf3).await?;
        Ok((cnf3 & 0x40) != 0)
    }

    pub async fn start_of_frame_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let cnf3 = self.read_register(Register::Cnf3).await?;
        Ok((cnf3 & 0x80) != 0)
    }
}
