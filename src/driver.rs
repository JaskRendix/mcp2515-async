use crate::can;
use crate::config::{AcceptanceFilter, BitrateConfig, FilterMask, TxPriority};
use crate::error::Error;
use crate::registers::{Instruction, OperationMode, Register};

use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

// --- Extracted Register Bit Masks & Flags ---

// CANCTRL / CANSTAT modes & bits
const CANCTRL_REQOP_MASK: u8 = 0xE0;
const CANCTRL_OSM: u8 = 0x08;
const CANCTRL_CLKOUT_EN: u8 = 0x04;
const CANCTRL_CLKOUT_DIV_MASK: u8 = 0x03;
const CANCTRL_ABAT: u8 = 0x10;

// TXBnCTRL flags
const TXB_TXREQ: u8 = 0x08;
const TXB_TXERR: u8 = 0x10;
const TXB_MLOA: u8 = 0x20;
const TXB_ABTF: u8 = 0x40;
const TXB_PRIORITY_MASK: u8 = 0x03;

// RXBn / Frame flags (Datasheet-aligned)
const SIDL_SRR: u8 = 0x10; // Standard Frame RTR bit in SIDL
const DLC_RTR: u8 = 0x40; // Extended Frame RTR bit in DLC
const RXB_ROLLOVER: u8 = 0x04;

// CANINTF / CANINTE flags
const INTF_RX0IF: u8 = 0x01;
const INTF_RX1IF: u8 = 0x02;
const INTF_TX0IF: u8 = 0x04;
const INTF_TX1IF: u8 = 0x08;
const INTF_TX2IF: u8 = 0x10;
const INTF_ERRIF: u8 = 0x20;
const INTF_WAKIF: u8 = 0x40;
const INTF_MERRIF: u8 = 0x80;
const INTF_TX_MASK: u8 = INTF_TX0IF | INTF_TX1IF | INTF_TX2IF;
const INTF_RX_MASK: u8 = INTF_RX0IF | INTF_RX1IF;

// EFLG flags
const EFLG_EWARN: u8 = 0x01;
const EFLG_RXWAR: u8 = 0x02;
const EFLG_TXWAR: u8 = 0x04;
const EFLG_RXEP: u8 = 0x08;
const EFLG_TXEP: u8 = 0x10;
const EFLG_TXBO: u8 = 0x20;
const EFLG_RX0OVR: u8 = 0x40;
const EFLG_RX1OVR: u8 = 0x80;

// CNF3 flags
const CNF3_WAKFIL: u8 = 0x40;
const CNF3_SOF: u8 = 0x80;

// --- ID Encoding/Decoding Helpers ---

struct IdRegisters {
    pub sidh: u8,
    pub sidl: u8,
    pub eid8: u8,
    pub eid0: u8,
}

fn encode_id(is_extended: bool, id: u32) -> IdRegisters {
    if is_extended {
        let sidh = (id >> 21) as u8;
        let sidl = (((id >> 13) & 0xE0) as u8) | 0x08 | (((id >> 16) & 0x03) as u8);
        let eid8 = (id >> 8) as u8;
        let eid0 = id as u8;
        IdRegisters {
            sidh,
            sidl,
            eid8,
            eid0,
        }
    } else {
        let sidh = (id >> 3) as u8;
        let sidl = ((id & 0x07) << 5) as u8;
        IdRegisters {
            sidh,
            sidl,
            eid8: 0,
            eid0: 0,
        }
    }
}

fn decode_id(sidh: u8, sidl: u8, eid8: u8, eid0: u8, dlc_byte: u8) -> (u32, bool, bool) {
    let is_extended = (sidl & 0x08) != 0;

    let is_rtr = if is_extended {
        (dlc_byte & DLC_RTR) != 0
    } else {
        (sidl & SIDL_SRR) != 0
    };

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

    (can_id, is_extended, is_rtr)
}

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
        self.modify_register(Register::CanCtrl, CANCTRL_REQOP_MASK, mode as u8)
            .await?;

        for _ in 0..100 {
            let stat = self.read_register(Register::CanStat).await?;

            if (stat & CANCTRL_REQOP_MASK) == (mode as u8) {
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
        let encoded = encode_id(is_extended, id);
        let buf = [
            Instruction::Write as u8,
            start_reg as u8,
            encoded.sidh,
            encoded.sidl,
            encoded.eid8,
            encoded.eid0,
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

    async fn find_free_tx_buffer(&mut self) -> Result<(Instruction, Instruction), Error<E, PinE>> {
        let txb0_ctrl = self.read_register(Register::TxB0Ctrl).await?;
        if (txb0_ctrl & TXB_TXREQ) == 0 {
            return Ok((Instruction::LoadTx0, Instruction::RtsTx0));
        }

        let txb1_ctrl = self.read_register(Register::TxB1Ctrl).await?;
        if (txb1_ctrl & TXB_TXREQ) == 0 {
            return Ok((Instruction::LoadTx1, Instruction::RtsTx1));
        }

        let txb2_ctrl = self.read_register(Register::TxB2Ctrl).await?;
        if (txb2_ctrl & TXB_TXREQ) == 0 {
            return Ok((Instruction::LoadTx2, Instruction::RtsTx2));
        }

        Err(Error::NoTxBufferAvailable)
    }

    pub async fn send(&mut self, frame: &can::CanFrame) -> Result<(), Error<E, PinE>> {
        if frame.can_dlc > 8 {
            return Err(Error::InvalidDlc);
        }

        let (load_instr, rts_instr) = self.find_free_tx_buffer().await?;

        let mut dlc_val = frame.can_dlc & 0x0F;

        if frame.is_rtr() {
            dlc_val |= DLC_RTR;
        }

        let is_ext = frame.is_extended();
        let id = if is_ext {
            frame.extended_id()
        } else {
            frame.standard_id() as u32
        };

        let encoded = encode_id(is_ext, id);
        let dlc_len = frame.can_dlc as usize;
        let mut buf = [0u8; 14];
        buf[0] = load_instr as u8;
        buf[1] = encoded.sidh;
        buf[2] = encoded.sidl;
        buf[3] = encoded.eid8;
        buf[4] = encoded.eid0;
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

        let (read_instr, clear_mask) = if (intf & INTF_RX0IF) != 0 {
            (Instruction::ReadRx0, INTF_RX0IF)
        } else if (intf & INTF_RX1IF) != 0 {
            (Instruction::ReadRx1, INTF_RX1IF)
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

        let (can_id, _, _) = decode_id(sidh, sidl, eid8, eid0, dlc_byte);
        let dlc = dlc_byte & 0x0F;

        if dlc > 8 {
            return Err(Error::InvalidDlc);
        }

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
        self.modify_register(reg, TXB_PRIORITY_MASK, priority as u8)
            .await
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
        Ok((eflg & EFLG_TXBO) != 0)
    }

    pub async fn is_error_warning(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & EFLG_EWARN) != 0)
    }

    pub async fn is_rx_warning(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & EFLG_RXWAR) != 0)
    }

    pub async fn is_tx_warning(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & EFLG_TXWAR) != 0)
    }

    pub async fn is_rx_error_passive(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & EFLG_RXEP) != 0)
    }

    pub async fn is_tx_error_passive(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & EFLG_TXEP) != 0)
    }

    pub async fn read_interrupt_flags(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::CanIntf).await
    }

    pub async fn read_interrupt_enable(&mut self) -> Result<u8, Error<E, PinE>> {
        self.read_register(Register::CanInte).await
    }

    pub async fn rx0_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & INTF_RX0IF) != 0)
    }

    pub async fn rx1_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & INTF_RX1IF) != 0)
    }

    pub async fn tx0_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & INTF_TX0IF) != 0)
    }

    pub async fn tx1_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & INTF_TX1IF) != 0)
    }

    pub async fn tx2_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & INTF_TX2IF) != 0)
    }

    pub async fn wakeup_interrupt_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & INTF_WAKIF) != 0)
    }

    pub async fn clear_wakeup_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanIntf, INTF_WAKIF, 0).await
    }

    pub async fn error_interrupt_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & INTF_ERRIF) != 0)
    }

    pub async fn enable_rx_interrupts(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_RX_MASK, INTF_RX_MASK)
            .await
    }

    pub async fn disable_rx_interrupts(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_RX_MASK, 0x00)
            .await
    }

    pub async fn clear_rx0_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanIntf, INTF_RX0IF, 0).await
    }

    pub async fn clear_rx1_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanIntf, INTF_RX1IF, 0).await
    }

    pub async fn clear_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanIntf, INTF_ERRIF, 0).await
    }

    pub async fn clear_interrupt_flags(&mut self) -> Result<(), Error<E, PinE>> {
        self.set_register(Register::CanIntf, 0x00).await
    }

    pub async fn enable_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_ERRIF, INTF_ERRIF)
            .await
    }

    pub async fn disable_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_ERRIF, 0x00)
            .await
    }

    pub async fn enable_wakeup_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_WAKIF, INTF_WAKIF)
            .await
    }

    pub async fn disable_wakeup_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_WAKIF, 0x00)
            .await
    }

    pub async fn enable_tx_interrupts(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_TX_MASK, INTF_TX_MASK)
            .await
    }

    pub async fn disable_tx_interrupts(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_TX_MASK, 0x00)
            .await
    }

    pub async fn rx0_overflowed(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & EFLG_RX0OVR) != 0)
    }

    pub async fn rx1_overflowed(&mut self) -> Result<bool, Error<E, PinE>> {
        let eflg = self.read_eflg().await?;
        Ok((eflg & EFLG_RX1OVR) != 0)
    }

    pub async fn clear_rx0_overflow(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Eflg, EFLG_RX0OVR, 0).await
    }

    pub async fn clear_rx1_overflow(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Eflg, EFLG_RX1OVR, 0).await
    }

    pub async fn clear_overflows(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Eflg, EFLG_RX0OVR | EFLG_RX1OVR, 0)
            .await
    }

    pub async fn tx0_busy(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB0Ctrl).await?;
        Ok((ctrl & TXB_TXREQ) != 0)
    }

    pub async fn tx1_busy(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB1Ctrl).await?;
        Ok((ctrl & TXB_TXREQ) != 0)
    }

    pub async fn tx2_busy(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB2Ctrl).await?;
        Ok((ctrl & TXB_TXREQ) != 0)
    }

    pub async fn tx0_error(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB0Ctrl).await?;
        Ok((ctrl & TXB_TXERR) != 0)
    }

    pub async fn tx1_error(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB1Ctrl).await?;
        Ok((ctrl & TXB_TXERR) != 0)
    }

    pub async fn tx2_error(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB2Ctrl).await?;
        Ok((ctrl & TXB_TXERR) != 0)
    }

    pub async fn tx0_lost_arbitration(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB0Ctrl).await?;
        Ok((ctrl & TXB_MLOA) != 0)
    }

    pub async fn tx1_lost_arbitration(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB1Ctrl).await?;
        Ok((ctrl & TXB_MLOA) != 0)
    }

    pub async fn tx2_lost_arbitration(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB2Ctrl).await?;
        Ok((ctrl & TXB_MLOA) != 0)
    }

    pub async fn tx0_aborted(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB0Ctrl).await?;
        Ok((ctrl & TXB_ABTF) != 0)
    }

    pub async fn tx1_aborted(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB1Ctrl).await?;
        Ok((ctrl & TXB_ABTF) != 0)
    }

    pub async fn tx2_aborted(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::TxB2Ctrl).await?;
        Ok((ctrl & TXB_ABTF) != 0)
    }

    pub async fn abort_all_transmissions(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_ABAT, CANCTRL_ABAT)
            .await?;
        self.modify_register(Register::CanCtrl, CANCTRL_ABAT, 0x00)
            .await
    }

    pub async fn enable_one_shot(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_OSM, CANCTRL_OSM)
            .await
    }

    pub async fn disable_one_shot(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_OSM, 0x00)
            .await
    }

    pub async fn is_one_shot_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::CanCtrl).await?;
        Ok((ctrl & CANCTRL_OSM) != 0)
    }

    pub async fn enable_clkout(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_CLKOUT_EN, CANCTRL_CLKOUT_EN)
            .await
    }

    pub async fn disable_clkout(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_CLKOUT_EN, 0x00)
            .await
    }

    pub async fn set_clkout_div1(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_CLKOUT_DIV_MASK, 0x00)
            .await
    }

    pub async fn set_clkout_div2(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_CLKOUT_DIV_MASK, 0x01)
            .await
    }

    pub async fn set_clkout_div4(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_CLKOUT_DIV_MASK, 0x02)
            .await
    }

    pub async fn set_clkout_div8(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanCtrl, CANCTRL_CLKOUT_DIV_MASK, 0x03)
            .await
    }

    pub async fn enable_rollover(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::RxB0Ctrl, RXB_ROLLOVER, RXB_ROLLOVER)
            .await
    }

    pub async fn disable_rollover(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::RxB0Ctrl, RXB_ROLLOVER, 0x00)
            .await
    }

    pub async fn read_status(&mut self) -> Result<u8, Error<E, PinE>> {
        let tx = [Instruction::ReadStatus as u8, 0x00];
        let mut rx = [0u8; 2];

        self.spi.transfer(&mut rx, &tx).await.map_err(Error::Spi)?;

        Ok(rx[1])
    }

    pub async fn message_available(&mut self) -> Result<bool, Error<E, PinE>> {
        let status = self.read_status().await?;
        Ok((status & 0x03) != 0)
    }

    pub async fn rx_status(&mut self) -> Result<u8, Error<E, PinE>> {
        let tx = [Instruction::RxStatus as u8, 0x00];
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
            return Err(Error::BufferTooLarge);
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
            return Err(Error::BufferTooLarge);
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
        Ok((ctrl & RXB_ROLLOVER) != 0)
    }

    pub async fn clkout_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let ctrl = self.read_register(Register::CanCtrl).await?;
        Ok((ctrl & CANCTRL_CLKOUT_EN) != 0)
    }

    pub async fn interrupt_code(&mut self) -> Result<u8, Error<E, PinE>> {
        let stat = self.read_register(Register::CanStat).await?;
        Ok((stat >> 1) & 0x07)
    }

    pub async fn enable_wakeup_filter(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Cnf3, CNF3_WAKFIL, CNF3_WAKFIL)
            .await
    }

    pub async fn disable_wakeup_filter(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Cnf3, CNF3_WAKFIL, 0x00)
            .await
    }

    pub async fn enable_start_of_frame(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Cnf3, CNF3_SOF, CNF3_SOF)
            .await
    }

    pub async fn disable_start_of_frame(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::Cnf3, CNF3_SOF, 0x00).await
    }

    pub async fn wakeup_filter_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let cnf3 = self.read_register(Register::Cnf3).await?;
        Ok((cnf3 & CNF3_WAKFIL) != 0)
    }

    pub async fn start_of_frame_enabled(&mut self) -> Result<bool, Error<E, PinE>> {
        let cnf3 = self.read_register(Register::Cnf3).await?;
        Ok((cnf3 & CNF3_SOF) != 0)
    }

    pub async fn enable_message_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_MERRIF, INTF_MERRIF)
            .await
    }

    pub async fn disable_message_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanInte, INTF_MERRIF, 0)
            .await
    }

    pub async fn message_error_interrupt_pending(&mut self) -> Result<bool, Error<E, PinE>> {
        let intf = self.read_interrupt_flags().await?;
        Ok((intf & INTF_MERRIF) != 0)
    }

    pub async fn clear_message_error_interrupt(&mut self) -> Result<(), Error<E, PinE>> {
        self.modify_register(Register::CanIntf, INTF_MERRIF, 0)
            .await
    }
}
