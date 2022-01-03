use pb_jelly::Message;

use crate::CompactStr;

impl Message for CompactStr {
    fn compute_size(&self) -> usize {
        self.len()
    }

    fn serialize<W: pb_jelly::PbBufferWriter>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(self.as_bytes())?;
        Ok(())
    }

    fn deserialize<B: pb_jelly::PbBufferReader>(&mut self, r: &mut B) -> std::io::Result<()> {
        match CompactStr::from_utf8_buf(r) {
            Ok(compact) => {
                *self = compact;
                Ok(())
            }
            Err(_) => Err(std::io::ErrorKind::InvalidData.into()),
        }
    }
}
