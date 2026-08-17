#![no_std]

pub mod can;
pub mod registers;

use embedded_hal_async::spi::SpiDevice;
use can::CanFrame;
use registers::{Instruction, Register, OperationMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    Spi(E),
    AllTxBusy,
    InvalidDataLength,
}

pub struct Mcp2515<SPI> {
    spi: SPI,
}

impl<SPI, E> Mcp2515<SPI>
where
    SPI: SpiDevice<u8, Error = E>,
{
    pub const fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub async fn reset(&mut self) -> Result<(), Error<E>> {
        self.spi.write(&[Instruction::Reset as u8]).await.map_err(Error::Spi)
    }

    pub async fn set_mode(&mut self, mode: OperationMode) -> Result<(), Error<E>> {
        self.modify_register(Register::CanCtrl, 0xE0, mode as u8).await
    }

    pub async fn read_register(&mut self, reg: Register) -> Result<u8, Error<E>> {
        let buf = [Instruction::Read as u8, reg as u8, 0x00];
        let mut res = [0u8; 3];
        self.spi.transfer(&mut res, &buf).await.map_err(Error::Spi)?;
        Ok(res[2])
    }

    pub async fn set_register(&mut self, reg: Register, val: u8) -> Result<(), Error<E>> {
        let buf = [Instruction::Write as u8, reg as u8, val];
        self.spi.write(&buf).await.map_err(Error::Spi)
    }

    pub async fn modify_register(&mut self, reg: Register, mask: u8, val: u8) -> Result<(), Error<E>> {
        let buf = [Instruction::BitMod as u8, reg as u8, mask, val];
        self.spi.write(&buf).await.map_err(Error::Spi)
    }
}
