
pub struct RVLCodec {
    buffer: Vec<i32>,
    p_buffer: usize,
    nibbles_written: i32,
    word: i32,
}

impl RVLCodec {
    pub fn new() -> Self {
        RVLCodec {
            buffer: Vec::new(),
            p_buffer: 0,
            nibbles_written: 0,
            word: 0,
        }
    }

    pub fn compress_rvl(&mut self, input: &[u16], output: &mut Vec<u8>) -> usize {
        self.buffer.clear();
        self.p_buffer = 0;
        self.nibbles_written = 0;
        let mut previous: u16 = 0;
        let mut input_iter = input.iter().peekable();

        while let Some(&&_current) = input_iter.peek() {
            let mut zeros = 0;
            while let Some(&&0) = input_iter.peek() {
                input_iter.next();
                zeros += 1;
            }
            self.encode_vle(zeros);

            let mut nonzeros = 0;
            while let Some(&&current) = input_iter.peek() {
                if current == 0 {
                    break;
                }
                input_iter.next();
                nonzeros += 1;
            }
            self.encode_vle(nonzeros);

            for _ in 0..nonzeros {
                let current = input_iter.next().unwrap();
                let delta = *current as i32 - previous as i32;
                let positive = (delta << 1) ^ (delta >> 31);
                self.encode_vle(positive);
                previous = *current;
            }
        }

        if self.nibbles_written != 0 {
            self.buffer.push(self.word << 4 * (8 - self.nibbles_written));
        }

        output.clear();
        for &word in &self.buffer {
            output.extend_from_slice(&word.to_le_bytes());
        }

        output.len()
    }

    pub fn decompress_rvl(&mut self, input: &[u8], output: &mut Vec<u16>) -> usize {
        self.buffer.clear();
        self.p_buffer = 0;
        self.nibbles_written = 0;

        for chunk in input.chunks_exact(4) {
            let word = i32::from_le_bytes(chunk.try_into().unwrap());
            self.buffer.push(word);
        }

        let mut previous: u16 = 0;
        let mut output_index = 0;

        while output_index < output.len() {
            let zeros = self.decode_vle();
            for _ in 0..zeros {
                output.push(0);
                output_index += 1;
            }

            let nonzeros = self.decode_vle();
            for _ in 0..nonzeros {
                let positive = self.decode_vle();
                let delta = (positive >> 1) ^ -(positive & 1);
                let current = previous.wrapping_add(delta as u16);
                output.push(current);
                output_index += 1;
                previous = current;
            }
        }

        output_index
    }

    fn encode_vle(&mut self, mut value: i32) {
        while value >= 0xF {
            self.write_nibble((value & 0xF) as u8);
            value >>= 4;
        }
        self.write_nibble(value as u8);
    }

    fn decode_vle(&mut self) -> i32 {
        let mut result = 0;
        let mut shift = 0;
        loop {
            let nibble = self.read_nibble();
            result |= (nibble as i32) << shift;
            if nibble < 0xF {
                break;
            }
            shift += 4;
        }
        result
    }

    fn write_nibble(&mut self, nibble: u8) {
        self.word |= (nibble as i32) << (4 * (self.nibbles_written as i32));
        self.nibbles_written += 1;
        if self.nibbles_written == 8 {
            self.buffer.push(self.word);
            self.nibbles_written = 0;
            self.word = 0;
        }
    }

    fn read_nibble(&mut self) -> u8 {
        if self.nibbles_written == 0 {
            if self.p_buffer < self.buffer.len() {
                self.word = self.buffer[self.p_buffer];
                self.p_buffer += 1;
            }
            self.nibbles_written = 8;
        }
        let nibble = (self.word & 0xF) as u8;
        self.word >>= 4;
        self.nibbles_written -= 1;
        nibble
    }
}

