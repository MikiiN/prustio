pub const NAME: &str = "wrapper.h";

pub const CONTENT: &str = r#"
#pragma once
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// --- Constants ---
#define C_INPUT 0x0
#define C_OUTPUT 0x1
#define C_INPUT_PULLUP 0x2
#define C_LOW 0x0
#define C_HIGH 0x1

// Interrupt Modes
#define C_CHANGE 1
#define C_FALLING 2
#define C_RISING 3

// Bit Order
#define C_LSBFIRST 0
#define C_MSBFIRST 1

void c_init();

// --- Core I/O ---
void c_pin_mode(uint8_t pin, uint8_t mode);
void c_digital_write(uint8_t pin, uint8_t val);
int c_digital_read(uint8_t pin);

void c_analog_reference(uint8_t mode);
void c_analog_write(uint8_t pin, int val);
int c_analog_read(uint8_t pin);

// --- Time ---
unsigned long c_millis(void);
unsigned long c_micros(void);
void c_delay(unsigned long ms);
void c_delay_microseconds(unsigned int us);

// --- Advanced I/O ---
void c_tone(uint8_t pin, unsigned int frequency, unsigned long duration);
void c_no_tone(uint8_t pin);
void c_shift_out(uint8_t data_pin, uint8_t clock_pin, uint8_t bit_order, uint8_t val);
uint8_t c_shift_in(uint8_t data_pin, uint8_t clock_pin, uint8_t bit_order);
unsigned long c_pulse_in(uint8_t pin, uint8_t state, unsigned long timeout);

// --- Math & Random ---
long c_map(long value, long fromLow, long fromHigh, long toLow, long toHigh);
long c_constrain(long x, long a, long b);
void c_random_seed(unsigned long seed);
long c_random_max(long max);
long c_random_range(long min, long max);

// --- Interrupts ---
void c_attach_interrupt(uint8_t interruptNum, void (*userFunc)(void), int mode);
void c_detach_interrupt(uint8_t interruptNum);
void c_interrupts(void);
void c_no_interrupts(void);

// --- Serial (UART) ---
void c_serial_begin(unsigned long baudRate);
void c_serial_end(void);
int c_serial_available(void);
int c_serial_read(void);
int c_serial_peek(void);
void c_serial_flush(void);
size_t c_serial_write(uint8_t val);
size_t c_serial_write_buffer(const uint8_t *buf, size_t len);
size_t c_serial_print_str(const char *str);
size_t c_serial_println_str(const char *str);
size_t c_serial_print_int(int val);
size_t c_serial_println_int(int val);

// --- Wire (I2C) ---
void c_wire_begin(void);
void c_wire_begin_slave(uint8_t address);
uint8_t c_wire_request_from(uint8_t address, uint8_t quantity, bool stop);
void c_wire_begin_transmission(uint8_t address);
uint8_t c_wire_end_transmission(bool stop);
size_t c_wire_write(uint8_t value);
size_t c_wire_write_buffer(const uint8_t *data, size_t quantity);
int c_wire_available(void);
int c_wire_read(void);
void c_wire_on_receive(void (*function)(int));
void c_wire_on_request(void (*function)(void));

// --- SPI ---
void c_spi_begin(void);
void c_spi_end(void);
void c_spi_begin_transaction(uint32_t clockSpeed, uint8_t bitOrder, uint8_t dataMode);
void c_spi_end_transaction(void);
uint8_t c_spi_transfer(uint8_t val);
uint16_t c_spi_transfer16(uint16_t val);
void c_spi_transfer_buffer(void *buf, size_t count);

#ifdef __cplusplus
}
#endif
"#;