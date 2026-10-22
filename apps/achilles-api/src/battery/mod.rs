//! Built for the Waveshare UPS Hat C
//! All info about the INA219 was found at
//! <https://www.waveshare.com/wiki/UPS_HAT_(C)>
pub mod worker;
use anyhow::Result;
pub use common::BatteryInfo;

use rpi_pal::i2c::I2c;

// INA219 register addresses
const REG_CONFIG: u8 = 0x00;
const REG_SHUNT_VOLTAGE: u8 = 0x01;
const REG_BUS_VOLTAGE: u8 = 0x02;
const REG_POWER: u8 = 0x03;
const REG_CURRENT: u8 = 0x04;
const REG_CALIBRATION: u8 = 0x05;

// config values
const BUS_RANGE_16V: u16 = 0x00;
const GAIN_80MV: u16 = 0x01;
const ADC_12BIT_32S: u16 = 0x0D;
const MODE_SHUNT_AND_BUS_CONTINUOUS: u16 = 0x07;

const CALIBRATION: u16 = 4096;
const CURRENT_LSB_A: f64 = 0.0001;
const POWER_LSB_W: f64 = 0.002;

pub struct Ina219 {
    pub i2c: I2c,
}

impl Ina219 {
    pub fn new(bus_number: u8, address: u16) -> Result<Self> {
        let mut i2c = I2c::with_bus(bus_number)?;
        i2c.set_slave_address(address)?;

        let mut sensor = Self { i2c };
        sensor.configure()?;

        Ok(sensor)
    }

    fn read_register(&mut self, register: u8) -> Result<u16> {
        let mut bytes = [0u8; 2];
        self.i2c.write_read(&[register], &mut bytes)?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn write_register(&mut self, register: u8, value: u16) -> Result<()> {
        let [high, low] = value.to_be_bytes();
        self.i2c.write(&[register, high, low])?;
        Ok(())
    }

    fn configure(&mut self) -> Result<()> {
        self.write_register(REG_CALIBRATION, CALIBRATION)?;

        let config = (BUS_RANGE_16V << 13)
            | (GAIN_80MV << 11)
            | (ADC_12BIT_32S << 7)
            | (ADC_12BIT_32S << 3)
            | MODE_SHUNT_AND_BUS_CONTINUOUS;

        self.write_register(REG_CONFIG, config)?;
        Ok(())
    }

    pub fn shunt_voltage_mv(&mut self) -> Result<f64> {
        let raw = self.read_register(REG_SHUNT_VOLTAGE)? as i16;
        Ok(f64::from(raw) * 0.01)
    }

    pub fn bus_voltage_v(&mut self) -> Result<f64> {
        let raw = self.read_register(REG_BUS_VOLTAGE)?;
        Ok(f64::from(raw >> 3) * 0.004)
    }

    pub fn current_a(&mut self) -> Result<f64> {
        let raw = self.read_register(REG_CURRENT)? as i16;
        Ok(f64::from(raw) * CURRENT_LSB_A)
    }

    pub fn power_w(&mut self) -> Result<f64> {
        let raw = self.read_register(REG_POWER)?;
        Ok(f64::from(raw) * POWER_LSB_W)
    }
}
