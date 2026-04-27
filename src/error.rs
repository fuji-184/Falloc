#[derive(Debug)]
pub enum Error {
    InvalidAlignment(&'static str),
    IntegerOverflow(&'static str),
    LayoutError(std::alloc::LayoutError),
    OutOfMemory
}