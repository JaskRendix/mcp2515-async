use core::convert::Infallible;
use embedded_hal_async::spi::{ErrorType, Operation, SpiDevice};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct MockSpi {
    state: Arc<Mutex<MockSpiState>>,
}

#[derive(Debug)]
pub struct MockSpiState {
    pub writes: Vec<Vec<u8>>,
    pub transfers: Vec<(Vec<u8>, Vec<u8>)>,
    pub next_transfer_output: Vec<u8>,
}

impl MockSpi {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockSpiState {
                writes: Vec::new(),
                transfers: Vec::new(),
                next_transfer_output: vec![0, 0, 0],
            })),
        }
    }

    pub fn with_transfer_output(self, out: &[u8]) -> Self {
        self.state.lock().unwrap().next_transfer_output = out.to_vec();
        self
    }

    pub fn writes(&self) -> Vec<Vec<u8>> {
        self.state.lock().unwrap().writes.clone()
    }

    pub fn transfers(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.state.lock().unwrap().transfers.clone()
    }
}

// REQUIRED BY embedded-hal-async
impl ErrorType for MockSpi {
    type Error = Infallible;
}

// REQUIRED BY embedded-hal-async SpiDevice
impl SpiDevice<u8> for MockSpi {
    async fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.state.lock().unwrap().writes.push(data.to_vec());
        Ok(())
    }

    async fn transfer(&mut self, out: &mut [u8], input: &[u8]) -> Result<(), Self::Error> {
        let mut state = self.state.lock().unwrap();
        state.transfers.push((out.to_vec(), input.to_vec()));
        out.copy_from_slice(&state.next_transfer_output);
        Ok(())
    }

    async fn transaction(&mut self, ops: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        let mut state = self.state.lock().unwrap();
        for op in ops {
            match op {
                Operation::Write(data) => {
                    state.writes.push(data.to_vec());
                }
                Operation::Transfer(read, write) => {
                    state.transfers.push((read.to_vec(), write.to_vec()));
                    read.copy_from_slice(&state.next_transfer_output);
                }
                _ => {} // Ignore other operation types not used by the driver
            }
        }
        Ok(())
    }
}
