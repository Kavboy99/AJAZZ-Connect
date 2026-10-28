//! Minimal PE reader for the RCDATA resources of vendor updaters. Bounds are checked on every
//! read; ambiguous structures are refused rather than guessed.

use super::InspectError;
use super::image::{slice, u16_at, u32_at};
use std::collections::BTreeMap;

const RT_RCDATA: u32 = 10;
const DIRECTORY: u32 = 0x8000_0000;

fn bad(detail: &str) -> InspectError {
    InspectError::malformed(detail)
}

pub(super) struct Pe<'a> {
    data: &'a [u8],
    sections: Vec<(usize, usize, usize)>,
    tree: &'a [u8],
}

impl<'a> Pe<'a> {
    pub(super) fn new(data: &'a [u8]) -> Result<Self, InspectError> {
        if !data.starts_with(b"MZ") {
            return Err(bad("PE expected"));
        }
        let nt = u32_at(data, 0x3c)? as usize;
        if slice(data, nt, 4)? != b"PE\0\0" {
            return Err(bad("damaged PE header"));
        }
        let count = u16_at(data, nt + 6)? as usize;
        if !(1..=96).contains(&count) {
            return Err(bad("section table size"));
        }
        let size = u16_at(data, nt + 20)? as usize;
        let opt = nt + 24;
        slice(data, opt, size)?;
        let dd = match u16_at(data, opt)? {
            0x10b => 96,
            0x20b => 112,
            _ => return Err(bad("unknown PE format")),
        };
        if size < dd + 24 {
            return Err(bad("no resource directory"));
        }
        let mut sections = Vec::with_capacity(count);
        for i in 0..count {
            let row = slice(data, opt + size + i * 40, 40)?;
            let rva = u32_at(row, 12)? as usize;
            let bytes = u32_at(row, 16)? as usize;
            let offset = u32_at(row, 20)? as usize;
            slice(data, offset, bytes)?;
            sections.push((rva, bytes, offset));
        }
        let mut pe = Self {
            data,
            sections,
            tree: &[],
        };
        let rva = u32_at(data, opt + dd + 16)? as usize;
        let bytes = u32_at(data, opt + dd + 20)? as usize;
        if bytes == 0 {
            return Err(bad("EXE has no resources"));
        }
        pe.tree = pe.rva(rva, bytes)?;
        Ok(pe)
    }

    fn rva(&self, address: usize, bytes: usize) -> Result<&'a [u8], InspectError> {
        let mut matches = Vec::new();
        for &(start, size, offset) in &self.sections {
            if let Some(delta) = address.checked_sub(start)
                && delta <= size
                && bytes <= size - delta
            {
                matches.push(slice(self.data, offset + delta, bytes)?);
            }
        }
        match matches.as_slice() {
            [only] => Ok(only),
            _ => Err(bad("invalid or ambiguous resource address")),
        }
    }

    fn entries(&self, at: usize) -> Result<Vec<(u32, u32)>, InspectError> {
        let count = u16_at(self.tree, at + 12)? as usize + u16_at(self.tree, at + 14)? as usize;
        if count > 4096 {
            return Err(bad("resource table too large"));
        }
        (0..count)
            .map(|i| {
                let row = slice(self.tree, at + 16 + i * 8, 8)?;
                Ok((u32_at(row, 0)?, u32_at(row, 4)?))
            })
            .collect()
    }

    /// RCDATA resources by numeric ID; exactly one language per resource.
    pub(super) fn rcdata(&self) -> Result<BTreeMap<u32, &'a [u8]>, InspectError> {
        let roots: Vec<_> = self
            .entries(0)?
            .into_iter()
            .filter(|(id, _)| *id == RT_RCDATA)
            .collect();
        let [(_, root)] = roots.as_slice() else {
            return Err(bad("no unique RCDATA"));
        };
        if root & DIRECTORY == 0 {
            return Err(bad("no unique RCDATA"));
        }
        let mut result = BTreeMap::new();
        for (id, directory) in self.entries((root & !DIRECTORY) as usize)? {
            if id & DIRECTORY != 0 || directory & DIRECTORY == 0 {
                return Err(bad("unknown resource tree"));
            }
            let language = self.entries((directory & !DIRECTORY) as usize)?;
            let [(_, leaf)] = language.as_slice() else {
                return Err(bad("ambiguous resource language"));
            };
            if leaf & DIRECTORY != 0 {
                return Err(bad("ambiguous resource language"));
            }
            let item = slice(self.tree, *leaf as usize, 16)?;
            let value = self.rva(u32_at(item, 0)? as usize, u32_at(item, 4)? as usize)?;
            if result.insert(id, value).is_some() {
                return Err(bad("duplicate RCDATA"));
            }
        }
        Ok(result)
    }
}
