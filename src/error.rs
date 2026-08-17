#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E, PinE = core::convert::Infallible> {
    Spi(E),
    Pin(PinE),
    AllTxBusy,
    InvalidDataLength,
}
