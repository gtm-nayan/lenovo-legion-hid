#![feature(assert_matches)]
#![feature(exclusive_range_pattern)]

use hidapi::{HidApi, HidDevice};

mod error;

#[cfg(target_os = "linux")]
const DEVICE_INFO_2021: (u16, u16, u16, u16) = (0x048d, 0xc965, 0, 0);
#[cfg(target_os = "linux")]
const DEVICE_INFO_2020: (u16, u16, u16, u16) = (0x048d, 0xc955, 0, 0);
#[cfg(target_os = "windows")]
const DEVICE_INFO_2021: (u16, u16, u16, u16) = (0x048d, 0xc965, 0xff89, 0x00cc);
#[cfg(target_os = "windows")]
const DEVICE_INFO_2020: (u16, u16, u16, u16) = (0x048d, 0xc955, 0xff89, 0x00cc);

const SPEED_RANGE: std::ops::RangeInclusive<u8> = 1..=4;
const BRIGHTNESS_RANGE: std::ops::RangeInclusive<u8> = 1..=2;

pub struct LightingState {
	speed: u8,
	brightness: u8,
	rgb_values: [u8; 12],
}

pub struct Keyboard {
	keyboard_hid: HidDevice,
	current_state: LightingState,
}

#[allow(dead_code)]
impl Keyboard {
	fn build_payload(&self) -> Result<[u8; 33], &'static str> {
		let keyboard_state = &self.current_state;

		if !SPEED_RANGE.contains(&keyboard_state.speed) {
			return Err("Speed is outside valid range (1-4)");
		}
		if !BRIGHTNESS_RANGE.contains(&keyboard_state.brightness) {
			return Err("Brightness is outside valid range (1-2)");
		}
		let mut payload: [u8; 33] = [0; 33];
		payload[0] = 0xcc;
		payload[1] = 0x16;
		payload[2] = 0x01;
		payload[3] = keyboard_state.speed;
		payload[4] = keyboard_state.brightness;
		payload[5..(5 + 12)].copy_from_slice(&keyboard_state.rgb_values);

		Ok(payload)
	}

	pub fn refresh(&mut self) {
		let payload = match self.build_payload() {
			Ok(payload) => payload,
			Err(err) => panic!("Payload build error: {}", err),
		};
		match self.keyboard_hid.send_feature_report(&payload) {
			Ok(_keyboard_hid) => {}
			Err(err) => panic!("Sending feature report failed: {}", err),
		};
	}

	pub fn set_speed(&mut self, speed: u8) {
		let speed = speed.clamp(*SPEED_RANGE.start(), *SPEED_RANGE.end());
		self.current_state.speed = speed;
		self.refresh();
	}

	pub fn set_brightness(&mut self, brightness: u8) {
		let brightness = brightness.clamp(*BRIGHTNESS_RANGE.start(), *BRIGHTNESS_RANGE.end());
		self.current_state.brightness = brightness;
		self.refresh();
	}

	pub fn set_colors_to(&mut self, new_values: &[u8; 12]) {
		self.current_state.rgb_values = *new_values;
		self.refresh();
	}
}

pub fn get_keyboard() -> Result<Keyboard, error::Error> {
	let api: HidApi = HidApi::new()?;

	let info = api
		.device_list()
		.find(|d| {
			let info_tuple = (d.vendor_id(), d.product_id(), d.usage_page(), d.usage());
			info_tuple == DEVICE_INFO_2021 || info_tuple == DEVICE_INFO_2020
		})
		.ok_or(error::Error::DeviceNotFound)?;

	let keyboard_hid: HidDevice = info.open_device(&api)?;

	let current_state: LightingState = LightingState {
		speed: 1,
		brightness: 1,
		rgb_values: [0; 12],
	};

	let mut keyboard = Keyboard {
		keyboard_hid,
		current_state,
	};

	keyboard.refresh();
	Ok(keyboard)
}
