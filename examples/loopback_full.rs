use core::convert::Infallible;
use embedded_hal_async::spi::{ErrorType, Operation, SpiDevice};
use mcp2515_async::{
    Bitrate, BitrateConfig, Clock, Mcp2515,
    registers::{Instruction, OperationMode},
};

// Minimal CAN frame
#[derive(Debug, Clone, Copy)]
struct CanFrame {
    id: u16,
    dlc: u8,
    data: [u8; 8],
}

// Example SPI mock
#[derive(Debug)]
struct MockSpi {
    writes: Vec<Vec<u8>>,
    transfers: Vec<(Vec<u8>, Vec<u8>)>,
    next_transfer_output: Vec<u8>,
}

impl MockSpi {
    fn new() -> Self {
        Self {
            writes: Vec::new(),
            transfers: Vec::new(),
            next_transfer_output: vec![0; 16],
        }
    }
}

impl ErrorType for MockSpi {
    type Error = Infallible;
}

impl SpiDevice<u8> for MockSpi {
    async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.writes.push(data.to_vec());
        Ok(())
    }

    async fn transfer(&mut self, out: &mut [u8], input: &[u8]) -> Result<(), Self::Error> {
        self.transfers.push((out.to_vec(), input.to_vec()));
        out.copy_from_slice(&self.next_transfer_output[..out.len()]);
        Ok(())
    }

    async fn transaction(&mut self, ops: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        for op in ops {
            match op {
                Operation::Write(data) => self.writes.push(data.to_vec()),
                Operation::Transfer(read, write) => {
                    self.transfers.push((read.to_vec(), write.to_vec()));
                    read.copy_from_slice(&self.next_transfer_output[..read.len()]);
                }
                _ => {} // Catch-all for Read, TransferInPlace, DelayNs, etc.
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    // Keep a handle to the mock SPI so we can inspect/drive it directly
    let mut spi = MockSpi::new();
    let mut can = Mcp2515::new(&mut spi);

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

    println!("Switching to Loopback mode...");
    can.set_mode(OperationMode::Loopback)
        .await
        .map_err(|e| format!("{:?}", e))?;

    println!("Loopback mode active.");

    // Build a frame
    let frame = CanFrame {
        id: 0x123,
        dlc: 3,
        data: [0xDE, 0xAD, 0xBE, 0, 0, 0, 0, 0],
    };

    // Encode into MCP2515 TX buffer format
    let tx_buf = [
        Instruction::LoadTx0 as u8,
        (frame.id >> 3) as u8,
        ((frame.id & 0x07) << 5) as u8,
        0,
        0,
        frame.dlc,
        frame.data[0],
        frame.data[1],
        frame.data[2],
    ];

    // Send TX buffer via mock SPI directly
    spi.write(&tx_buf).await.unwrap();

    // RTS
    spi.write(&[Instruction::RtsTx0 as u8]).await.unwrap();

    println!("Frame sent.");

    // Simulate RX buffer content
    spi.next_transfer_output = vec![
        0,
        0,
        0,
        (frame.id >> 3) as u8,
        ((frame.id & 0x07) << 5) as u8,
        0,
        0,
        frame.dlc,
        frame.data[0],
        frame.data[1],
        frame.data[2],
        0, // Padding byte 12
        0, // Padding byte 13
    ];

    let mut rx_buf = [0u8; 13];
    spi.transfer(&mut rx_buf, &[Instruction::ReadRx0 as u8])
        .await
        .unwrap();

    let rx_id = ((rx_buf[3] as u16) << 3) | ((rx_buf[4] >> 5) as u16);
    let rx_dlc = rx_buf[7];
    let rx_data = [rx_buf[8], rx_buf[9], rx_buf[10]];

    println!("Received frame:");
    println!("ID   = 0x{:03X}", rx_id);
    println!("DLC  = {}", rx_dlc);
    println!(
        "DATA = {:02X} {:02X} {:02X}",
        rx_data[0], rx_data[1], rx_data[2]
    );

    Ok(())
}
