use std::io;
use std::io::{Read, Write};

pub struct MockTcpStream {
    pub read_data: Vec<u8>,
    pub position: usize,
    pub write_data: Vec<u8>,
}

impl Read for MockTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.read_data.len() - self.position;
        let to_copy = std::cmp::min(remaining, buf.len());
        buf[..to_copy].copy_from_slice(&self.read_data[self.position..self.position + to_copy]);
        self.position += to_copy;
        Ok(to_copy)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let remaining = self.read_data.len() - self.position;
        if buf.len() > remaining {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "not enough data in MockTcpStream",
            ));
        }

        buf.copy_from_slice(&self.read_data[self.position..self.position + buf.len()]);
        self.position += buf.len();
        Ok(())
    }
}

impl Write for MockTcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
