use std::fs::File;
use std::io::{Write, BufWriter};

pub struct WavWriter {
    writer: BufWriter<File>,
    sample_count: u32,
    sample_rate: u32,
}

impl WavWriter {
    pub fn new(filename: &str, sample_rate: u32) -> std::io::Result<Self> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);
        
        // Write WAV header (we'll update it later)
        writer.write_all(b"RIFF")?;
        writer.write_all(&[0; 4])?; // File size - 8 (placeholder)
        writer.write_all(b"WAVE")?;
        
        // Format chunk
        writer.write_all(b"fmt ")?;
        writer.write_all(&16u32.to_le_bytes())?; // Chunk size
        writer.write_all(&1u16.to_le_bytes())?;  // Audio format (PCM)
        writer.write_all(&2u16.to_le_bytes())?;  // Channels (stereo)
        writer.write_all(&sample_rate.to_le_bytes())?;
        writer.write_all(&(sample_rate * 2 * 2).to_le_bytes())?; // Byte rate
        writer.write_all(&4u16.to_le_bytes())?;  // Block align
        writer.write_all(&16u16.to_le_bytes())?; // Bits per sample
        
        // Data chunk header
        writer.write_all(b"data")?;
        writer.write_all(&[0; 4])?; // Data size (placeholder)
        
        Ok(WavWriter {
            writer,
            sample_count: 0,
            sample_rate,
        })
    }
    
    pub fn write_sample(&mut self, left: f32, right: f32) -> std::io::Result<()> {
        // Convert f32 (-1.0 to 1.0) to i16
        let left_i16 = (left.clamp(-1.0, 1.0) * 32767.0) as i16;
        let right_i16 = (right.clamp(-1.0, 1.0) * 32767.0) as i16;
        
        self.writer.write_all(&left_i16.to_le_bytes())?;
        self.writer.write_all(&right_i16.to_le_bytes())?;
        self.sample_count += 1;
        
        Ok(())
    }
    
    pub fn finish(mut self) -> std::io::Result<()> {
        self.writer.flush()?;
        
        // Update file size and data size in header
        let mut file = self.writer.into_inner()?;
        let data_size = self.sample_count * 2 * 2; // samples * channels * bytes_per_sample
        let file_size = data_size + 36; // header size
        
        use std::io::Seek;
        file.seek(std::io::SeekFrom::Start(4))?;
        file.write_all(&file_size.to_le_bytes())?;
        file.seek(std::io::SeekFrom::Start(40))?;
        file.write_all(&data_size.to_le_bytes())?;
        
        Ok(())
    }
}