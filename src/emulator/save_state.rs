use crate::emulator::Emulator;
use crate::serializable::Serializable;
use std::io::{Cursor, Error, ErrorKind, Result};

pub struct SaveStateHeader {
    pub version: u8,
    pub title: String,
}

pub const MAJOR_VERSION: u8 = 7;
pub const HEADER_IDENTIFIER: &str = "HEADER";
pub const STATE_IDENTIFIER: &str = "STATE";
pub const FORMAT_ERROR: &str = "The provided save state file is in an invalid format.";

fn as_invalid_data_result(message: &str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}

fn as_format_error_result(message: &str) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{} Error: {}", FORMAT_ERROR, message))
}

impl Emulator {
    pub fn encode_save_state(&self) -> Result<Vec<u8>> {
        let mut save_state_bytes = Vec::new();

        let header_identifier_bytes = HEADER_IDENTIFIER.as_bytes();
        save_state_bytes.extend_from_slice(header_identifier_bytes);

        save_state_bytes.push(MAJOR_VERSION);
        let title = self.address_bus().cartridge_mapper().title();
        save_state_bytes.push(title.len() as u8);
        save_state_bytes.extend_from_slice(title.as_bytes());

        let state_identifier_bytes = STATE_IDENTIFIER.as_bytes();
        save_state_bytes.extend_from_slice(state_identifier_bytes);

        let mut emulator_bytes = Vec::new();
        match self.serialize(&mut emulator_bytes) {
            Ok(()) => {
                save_state_bytes.extend_from_slice(&emulator_bytes);
                Ok(save_state_bytes)
            },
            Err(e) => Err(as_invalid_data_result(e.to_string().as_str()))
        }
    }

    pub fn apply_save_state(&mut self, data: &[u8]) -> Result<()> {
        let header_identifier_size = HEADER_IDENTIFIER.len();
        let header_identifier_bytes = &data[..header_identifier_size];
        if data.len() < header_identifier_size || header_identifier_bytes != HEADER_IDENTIFIER.as_bytes() {
            Err(as_format_error_result("Invalid save state header."))
        }
        else {
            let version = data[header_identifier_size];
            let title_length = data[header_identifier_size + 1] as usize;
            let title_start = header_identifier_size + 2;
            let state_identifier_start = title_start + title_length;
            let title_bytes = data[title_start..state_identifier_start].to_vec();
            let title = String::from_utf8(title_bytes)
                .map_err(|e| as_invalid_data_result(e.to_string().as_str()))?;
        
            let header = SaveStateHeader { version, title };
        
            let state_identifier_size = STATE_IDENTIFIER.len();
            let state_start = state_identifier_start + state_identifier_size;
            let state_identifier_bytes= &data[state_identifier_start..state_start];
        
            let current_game_title = self.address_bus().cartridge_mapper().title();

            if state_start > data.len() || state_identifier_bytes != STATE_IDENTIFIER.as_bytes() {
                Err(as_format_error_result("Invalid save state identifier."))
            }
            else if version != MAJOR_VERSION {
                Err(as_format_error_result(format!("The save state is using an older incompatible version. Save state version: {}, Current version: {}.", header.version, MAJOR_VERSION).as_str()))
            }
            else if header.title != current_game_title {
                Err(as_format_error_result(format!("This save state is meant to be used for a different game. Save state game key: '{}', Current game key: '{}'.", header.title, current_game_title).as_str()))
            }
            else {
                let state_bytes = &data[state_start..];
                let mut cursor = Cursor::new(state_bytes);
                self.deserialize(&mut cursor)
            }
        }
    }
}
