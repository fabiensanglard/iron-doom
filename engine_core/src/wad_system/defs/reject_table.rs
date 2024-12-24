/// Data structure attached to levels which is used to speed up line-of-sight
/// calculations.
#[derive(Default)]
pub struct WadRejectTable {
    lump_data: Vec<u8>,
    num_sectors: usize,
}

impl WadRejectTable {
    pub fn new(lump_data: &[u8], num_sectors: usize) -> Self {
        Self {
            lump_data: lump_data.to_vec(),
            num_sectors,
        }
    }

    pub fn is_rejected(&self, s1: usize, s2: usize) -> bool {
        let pnum = s1 * self.num_sectors + s2;
        let byte_num = pnum / 8;

        if let Some(val) = self.lump_data.get(byte_num) {
            let bit_num = 1usize << (pnum & 7);
            let reject_val = bit_num & *val as usize;
            return reject_val != 0;
        }

        false
    }
}
