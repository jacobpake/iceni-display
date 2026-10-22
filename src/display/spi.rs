use crate::display::volatile_cell::VolatileCell;

unsafe extern "Rust" {
    #[cheriot_mmio(name = "spi1", permissions = "RW")]
    pub static SPI1: SpiInterface;
}

#[derive(Debug)]
pub enum SpiError {
    TxStall,
    InvalidLength,
}

const MAX_SPI_COMMAND_LEN: usize = 1 << 20;

const CONTROL_OUTPUT_ENABLE: u32 = 1 << 29;
const CONTROL_SPI_ENABLE: u32 = 1 << 31;

const STATUS_READY: u32 = 1 << 31;
const STATUS_ACTIVE: u32 = 1 << 30;
const STATUS_TX_FULL: u32 = 1 << 29;
const STATUS_TX_EMPTY: u32 = 1 << 28;
const STATUS_TX_STALL: u32 = 1 << 27;

const STATUS_RX_EMPTY: u32 = 1 << 24;

const COMMAND_DIRECTION_TX: u32 = 0b10 << 3;
const COMMAND_SPEED_STD: u32 = 0b00 << 1;

#[repr(C)]
pub struct SpiInterface {
    pub interrupt_state: VolatileCell<u32>,        // 0x00
    pub interrupt_enable: VolatileCell<u32>,       // 0x04
    pub interrupt_test: VolatileCell<u32>,         // 0x08
    pub alert_test: VolatileCell<u32>,             // 0x0C
    pub control: VolatileCell<u32>,                // 0x10
    pub status: VolatileCell<u32>,                 // 0x14
    pub configuration: VolatileCell<u32>,          // 0x18
    pub chip_select_id: VolatileCell<u32>,         // 0x1C
    pub command: VolatileCell<u32>,                // 0x20
    pub receive_data: VolatileCell<u32>,           // 0x24
    pub transmit_data: VolatileCell<u32>,          // 0x28
    pub error_interrupt_enable: VolatileCell<u32>, // 0x2C
    pub error_status: VolatileCell<u32>,           // 0x30
    pub event_interrupt_enable: VolatileCell<u32>, // 0x34
}

pub struct Spi(*mut SpiInterface);

impl Spi {
    pub unsafe fn get() -> Self {
        Self((&raw const SPI1) as *mut SpiInterface)
    }

    pub fn status_wait(&self, required: u32, forbidden: u32) -> Result<(), SpiError> {
        let spi = unsafe { &*self.0 };
        loop {
            let status = spi.status.read();
            if status & required == required && status & forbidden == 0 {
                return Ok(());
            }
            if status & STATUS_TX_STALL != 0 {
                return Err(SpiError::TxStall);
            }
        }
    }

    pub fn init(
        &self,
        clock_divisor: u32,
        cs_idle: u32,
        cs_trail: u32,
        cs_lead: u32,
        clock_phase_trailing: bool,
        clock_polarity_low: bool,
    ) -> Result<(), SpiError> {
        let spi = unsafe { &*self.0 };
        spi.control.write(1 << 30);

        self.status_wait(STATUS_TX_EMPTY | STATUS_RX_EMPTY, 0)?;

        spi.control
            .write(CONTROL_SPI_ENABLE | CONTROL_OUTPUT_ENABLE);

        let configuration = clock_divisor
            | (cs_idle << 16)
            | (cs_trail << 20)
            | (cs_lead << 24)
            | if clock_phase_trailing { 1 << 30 } else { 0 }
            | if clock_polarity_low { 0 } else { 1 << 31 };

        spi.configuration.write(configuration);

        if let Err(err) = self.status_wait(STATUS_TX_EMPTY | STATUS_RX_EMPTY, 0) {
            return Err(err);
        }

        self.status_wait(STATUS_READY, 0)?;

        Ok(())
    }

    pub fn write(&self, data: &[u8]) {
        let res = self.blocking_write(data);
        if res.is_err() {
            eprintln!("err!");
        }
    }

    pub fn blocking_write(&self, data: &[u8]) -> Result<(), SpiError> {
        let len = data.len();

        if len == 0 || len > MAX_SPI_COMMAND_LEN {
            return Err(SpiError::InvalidLength);
        }

        let spi = unsafe { &*self.0 };

        spi.chip_select_id.write(0);

        self.status_wait(STATUS_READY, 0)?;

        let mut command_written = false;
        let command = COMMAND_DIRECTION_TX | COMMAND_SPEED_STD | (((len - 1) as u32) << 5);

        let mut offset = 0;
        let mut remaining = len;

        while remaining >= 4 {
            if spi.status.read() & STATUS_TX_FULL != 0 {
                if !command_written {
                    spi.command.write(command);
                    command_written = true;
                }

                self.status_wait(STATUS_READY, STATUS_TX_FULL)?;
            }

            let word = u32::from_ne_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);

            spi.transmit_data.write(word);
            offset += 4;
            remaining -= 4;
        }

        while remaining > 0 {
            if spi.status.read() & STATUS_TX_FULL != 0 {
                if !command_written {
                    spi.command.write(command);
                    command_written = true;
                }

                self.status_wait(STATUS_READY, STATUS_TX_FULL)?;
            }

            spi.transmit_data.write_other(data[offset]);
            offset += 1;
            remaining -= 1;
        }

        if !command_written {
            spi.command.write(command);
        }

        self.status_wait(STATUS_TX_EMPTY, STATUS_ACTIVE)?;

        Ok(())
    }
}
