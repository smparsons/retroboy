use crate::gpu::Gpu;

impl Gpu {
    fn should_trigger_oam_bug(&self, min_row: usize) -> bool {
        !self.cgb_mode
            && self.oam_scan_row != 0xFF
            && self.oam_scan_row >= 8
            && (self.oam_scan_row as usize) >= min_row
            && (self.oam_scan_row as usize) + 7 < self.object_attribute_memory.len()
    }

    fn read_u16_at(&self, offset: usize) -> u16 {
        u16::from_le_bytes([
            self.object_attribute_memory[offset],
            self.object_attribute_memory[offset + 1],
        ])
    }

    fn write_u16_at(&mut self, offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.object_attribute_memory[offset] = bytes[0];
        self.object_attribute_memory[offset + 1] = bytes[1];
    }

    fn copy_oam_bytes(&mut self, from_row: usize, to_row: usize, byte_range: std::ops::Range<usize>) {
        for i in byte_range {
            self.object_attribute_memory[to_row + i] = self.object_attribute_memory[from_row + i];
        }
    }

    pub(crate) fn trigger_oam_bug_write(&mut self) {
        if self.should_trigger_oam_bug(8) {
            let row = self.oam_scan_row as usize;
            let prev_row = row - 8;

            let a = self.read_u16_at(row);
            let b = self.read_u16_at(prev_row);
            let c = self.read_u16_at(prev_row + 4);

            let corrupted = ((a ^ c) & (b ^ c)) ^ c;
            self.write_u16_at(row, corrupted);
            self.copy_oam_bytes(prev_row, row, 2..8);
        }
    }

    pub(crate) fn trigger_oam_bug_read(&mut self) {
        if self.should_trigger_oam_bug(8) {
            let row = self.oam_scan_row as usize;
            let prev_row = row - 8;

            let a = self.read_u16_at(row);
            let b = self.read_u16_at(prev_row);
            let c = self.read_u16_at(prev_row + 4);

            let corrupted = b | (a & c);
            self.write_u16_at(row, corrupted);
            self.copy_oam_bytes(prev_row, row, 2..8);
        }
    }

    pub(crate) fn trigger_oam_bug_mixed(&mut self) {
        if self.should_trigger_oam_bug(32) {
            let row = self.oam_scan_row as usize;
            let prev_row = row - 8;
            let prev_prev_row = row - 16;

            let a = self.read_u16_at(prev_prev_row);
            let b = self.read_u16_at(prev_row);
            let c = self.read_u16_at(row);
            let d = self.read_u16_at(prev_row + 4);

            let corrupted = (b & (a | c | d)) | (a & c & d);
            self.write_u16_at(prev_row, corrupted);

            for i in 0..8 {
                let value = self.object_attribute_memory[prev_row + i];
                self.object_attribute_memory[row + i] = value;
                self.object_attribute_memory[prev_prev_row + i] = value;
            }

            self.trigger_oam_bug_read();
        }
    }
}
