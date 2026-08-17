use core::convert::Infallible;
use embedded_hal_async::spi::{ErrorType, Operation, SpiDevice};
use mcp2515_async::registers::{OperationMode, Register};
use mcp2515_async::{Bitrate, BitrateConfig, Clock, Mcp2515};

#[derive(Debug, Clone)]
struct MockSpi;

impl ErrorType for MockSpi {
    type Error = Infallible;
}

impl SpiDevice<u8> for MockSpi {
    async fn write(&mut self, _data: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn transfer(&mut self, out: &mut [u8], _input: &[u8]) -> Result<(), Self::Error> {
        out.fill(0);
        Ok(())
    }

    async fn transaction(&mut self, _ops: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let spi = MockSpi;
    let mut can = Mcp2515::new(spi);

    println!("Reset...");
    can.reset().await.map_err(|e| format!("{:?}", e))?;

    println!("Entering Config mode...");
    can.set_mode(OperationMode::Config)
        .await
        .map_err(|e| format!("{:?}", e))?;

    if let Some(cfg) = BitrateConfig::new(Clock::MHz16, Bitrate::Kbps500) {
        println!("Setting bitrate...");
        can.set_bitrate(cfg).await.map_err(|e| format!("{:?}", e))?;
    }

    let cnf1 = can.read_register(Register::Cnf1).await.unwrap();
    println!("CNF1 = 0x{:02X}", cnf1);

    println!("Switching to Loopback mode...");
    can.set_mode(OperationMode::Loopback)
        .await
        .map_err(|e| format!("{:?}", e))?;

    println!("Loopback mode active.");

    Ok(())
}
