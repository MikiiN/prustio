pub const NAME: &str = "wrapper.cpp";

pub const CONTENT: &str = r#"
#include <Arduino.h>
#include <Wire.h>
#include <SPI.h>
#include "wrapper.h"

#ifdef __cplusplus
extern "C" {
#endif

    void c_init() { init(); }

    // --- Core I/O ---
    void c_pin_mode(uint8_t pin, uint8_t mode) { pinMode(pin, mode); }
    void c_digital_write(uint8_t pin, uint8_t val) { digitalWrite(pin, val); }
    int c_digital_read(uint8_t pin) { return digitalRead(pin); }
    void c_analog_reference(uint8_t mode) { analogReference(mode); }
    void c_analog_write(uint8_t pin, int val) { analogWrite(pin, val); }
    int c_analog_read(uint8_t pin) { return analogRead(pin); }

    // --- Time ---
    unsigned long c_millis() { return millis(); }
    unsigned long c_micros() { return micros(); }
    void c_delay(unsigned long ms) { delay(ms); }
    void c_delay_microseconds(unsigned int us) { delayMicroseconds(us); }

    // --- Advanced I/O ---
    void c_tone(uint8_t pin, unsigned int frequency, unsigned long duration) { 
        if (duration > 0) tone(pin, frequency, duration);
        else tone(pin, frequency);
    }
    void c_no_tone(uint8_t pin) { noTone(pin); }
    void c_shift_out(uint8_t data_pin, uint8_t clock_pin, uint8_t bit_order, uint8_t val) { 
        shiftOut(data_pin, clock_pin, bit_order, val); 
    }
    uint8_t c_shift_in(uint8_t data_pin, uint8_t clock_pin, uint8_t bit_order) { 
        return shiftIn(data_pin, clock_pin, bit_order); 
    }
    unsigned long c_pulse_in(uint8_t pin, uint8_t state, unsigned long timeout) { 
        return pulseIn(pin, state, timeout); 
    }

    // --- Math & Random ---
    long c_map(long value, long fromLow, long fromHigh, long toLow, long toHigh) { 
        return map(value, fromLow, fromHigh, toLow, toHigh); 
    }
    long c_constrain(long x, long a, long b) { return constrain(x, a, b); }
    void c_random_seed(unsigned long seed) { randomSeed(seed); }
    long c_random_max(long max) { return random(max); }
    long c_random_range(long min, long max) { return random(min, max); }

    // --- Interrupts ---
    void c_attach_interrupt(uint8_t interruptNum, void (*userFunc)(void), int mode) { 
        attachInterrupt(digitalPinToInterrupt(interruptNum), userFunc, mode); 
    }
    void c_detach_interrupt(uint8_t interruptNum) { detachInterrupt(digitalPinToInterrupt(interruptNum)); }
    void c_interrupts() { interrupts(); }
    void c_no_interrupts() { noInterrupts(); }

    // --- Serial (UART) ---
    void c_serial_begin(unsigned long baudRate) { Serial.begin(baudRate); }
    void c_serial_end() { Serial.end(); }
    int c_serial_available() { return Serial.available(); }
    int c_serial_read() { return Serial.read(); }
    int c_serial_peek() { return Serial.peek(); }
    void c_serial_flush() { Serial.flush(); }
    size_t c_serial_write(uint8_t val) { return Serial.write(val); }
    size_t c_serial_write_buffer(const uint8_t *buf, size_t len) { return Serial.write(buf, len); }
    size_t c_serial_print_str(const char *str) { return Serial.print(str); }
    size_t c_serial_println_str(const char *str) { return Serial.println(str); }
    size_t c_serial_print_int(int val) { return Serial.print(val); }
    size_t c_serial_println_int(int val) { return Serial.println(val); }

    // --- Wire (I2C) ---
    void c_wire_begin() { Wire.begin(); }
    void c_wire_begin_slave(uint8_t address) { Wire.begin(address); }
    uint8_t c_wire_request_from(uint8_t address, uint8_t quantity, bool stop) { 
        return Wire.requestFrom(address, quantity, (uint8_t)stop); 
    }
    void c_wire_begin_transmission(uint8_t address) { Wire.beginTransmission(address); }
    uint8_t c_wire_end_transmission(bool stop) { return Wire.endTransmission((uint8_t)stop); }
    size_t c_wire_write(uint8_t value) { return Wire.write(value); }
    size_t c_wire_write_buffer(const uint8_t *data, size_t quantity) { return Wire.write(data, quantity); }
    int c_wire_available() { return Wire.available(); }
    int c_wire_read() { return Wire.read(); }
    void c_wire_on_receive(void (*function)(int)) { Wire.onReceive(function); }
    void c_wire_on_request(void (*function)(void)) { Wire.onRequest(function); }

    // --- SPI ---
    void c_spi_begin() { SPI.begin(); }
    void c_spi_end() { SPI.end(); }
    void c_spi_begin_transaction(uint32_t clockSpeed, uint8_t bitOrder, uint8_t dataMode) {
        SPI.beginTransaction(SPISettings(clockSpeed, bitOrder, dataMode));
    }
    void c_spi_end_transaction() { SPI.endTransaction(); }
    uint8_t c_spi_transfer(uint8_t val) { return SPI.transfer(val); }
    uint16_t c_spi_transfer16(uint16_t val) { return SPI.transfer16(val); }
    void c_spi_transfer_buffer(void *buf, size_t count) { SPI.transfer(buf, count); }

#ifdef __cplusplus
}
#endif

// ==============================================================================
// DUMMY FUNCTIONS TO SATISFY PLATFORMIO
// ==============================================================================
// PlatformIO requires these to successfully link its temporary firmware.elf.
// Because the final Rust application defines its own `main()` function, 
// the Rust linker will completely ignore Arduino's main.cpp, and these 
// dummy functions will be automatically stripped out of the final firmware!

void setup() {}
void loop() {}
"#;