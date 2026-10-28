//! HID report-descriptor walk for the standard Lighting and Illumination page (0x59). The field
//! order and widths are checked against the standard before any report is used. Written
//! independently from the USB HID usage tables and the Microsoft reference descriptor.

use crate::core::DriverError;
use std::collections::BTreeMap;

/// Report IDs of the LampArray reports and the MultiUpdate capacity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportMap {
    pub attributes: u8,
    pub request: u8,
    pub response: u8,
    pub multi: u8,
    pub range: u8,
    pub control: u8,
    pub capacity: usize,
}

#[derive(Clone, Copy, Default)]
struct Globals {
    page: u32,
    id: u8,
    bits: u32,
    count: u32,
}

type Fields = Vec<(u16, u32)>;

fn bad(detail: &str) -> DriverError {
    DriverError::protocol(detail)
}

/// The report layouts the standard prescribes, as (usage, bit width) in order.
fn expected() -> [(u16, Fields); 5] {
    [
        (
            2,
            vec![(3, 16), (4, 32), (5, 32), (6, 32), (7, 32), (8, 32)],
        ),
        (0x20, vec![(0x21, 16)]),
        (
            0x22,
            vec![
                (0x21, 16),
                (0x23, 32),
                (0x24, 32),
                (0x25, 32),
                (0x27, 32),
                (0x26, 32),
                (0x28, 8),
                (0x29, 8),
                (0x2a, 8),
                (0x2b, 8),
                (0x2c, 8),
                (0x2d, 8),
            ],
        ),
        (
            0x60,
            vec![
                (0x55, 8),
                (0x61, 16),
                (0x62, 16),
                (0x51, 8),
                (0x52, 8),
                (0x53, 8),
                (0x54, 8),
            ],
        ),
        (0x70, vec![(0x71, 8)]),
    ]
}

pub fn parse_descriptor(data: &[u8]) -> Result<ReportMap, DriverError> {
    if data.len() > 16384 {
        return Err(bad("descriptor too large"));
    }
    let mut at = 0;
    let mut g = Globals::default();
    let mut saved = Vec::new();
    let mut usages: Vec<u32> = Vec::new();
    let mut minimum = None;
    let mut collections: Vec<u32> = Vec::new();
    let mut reports: BTreeMap<u16, (u8, Fields)> = BTreeMap::new();
    let mut id_owner = BTreeMap::new();
    while at < data.len() {
        let tag = data[at];
        at += 1;
        if tag == 0xfe {
            return Err(bad("long HID items are not supported"));
        }
        let n = match tag & 3 {
            3 => 4,
            x => x as usize,
        };
        let raw = data
            .get(at..at + n)
            .ok_or_else(|| bad("truncated HID item"))?;
        at += n;
        let value = raw
            .iter()
            .enumerate()
            .fold(0u32, |a, (i, b)| a | ((*b as u32) << (i * 8)));
        let extended = |v: u32| if n == 4 { v } else { (g.page << 16) | v };
        match tag & 0xfc {
            0x04 => g.page = value,
            0x74 => g.bits = value,
            0x94 => g.count = value,
            0x84 => {
                if !(1..=255).contains(&value) {
                    return Err(bad("zero report ID"));
                }
                g.id = value as u8;
            }
            0xa4 => {
                if saved.len() >= 16 {
                    return Err(bad("HID push overflow"));
                }
                saved.push(g);
            }
            0xb4 => g = saved.pop().ok_or_else(|| bad("HID pop without push"))?,
            0x08 => usages.push(extended(value)),
            0x18 => minimum = Some(extended(value)),
            0x28 => {
                let first = minimum
                    .take()
                    .ok_or_else(|| bad("usage maximum without minimum"))?;
                let last = extended(value);
                if last < first || last - first >= 1024 {
                    return Err(bad("usage range too large"));
                }
                usages.extend(first..=last);
            }
            0xa0 => {
                if collections.len() >= 32 {
                    return Err(bad("collections nested too deep"));
                }
                collections.push(usages.first().copied().unwrap_or(0));
            }
            0xc0 => {
                collections
                    .pop()
                    .ok_or_else(|| bad("end collection without collection"))?;
            }
            0xb0 => {
                if g.count > 2048 || !(1..=32).contains(&g.bits) {
                    return Err(bad("unusual feature fields"));
                }
                let report_usage = collections.iter().rev().find(|u| {
                    **u >> 16 == 0x59
                        && matches!(**u & 0xffff, 2 | 0x20 | 0x22 | 0x50 | 0x60 | 0x70)
                });
                if !collections.contains(&0x590001) {
                    return Err(bad("feature outside a LampArray"));
                }
                let usage = (report_usage
                    .ok_or_else(|| bad("feature outside a LampArray report"))?
                    & 0xffff) as u16;
                if g.id == 0 || value & 2 == 0 {
                    return Err(bad("numbered variable feature reports are required"));
                }
                if id_owner.insert(g.id, usage).is_some_and(|old| old != usage) {
                    return Err(bad("report ID assigned to several reports"));
                }
                let entry = reports.entry(usage).or_insert((g.id, Vec::new()));
                let last = *usages.last().ok_or_else(|| bad("feature without usages"))?;
                if entry.0 != g.id {
                    return Err(bad("ambiguous LampArray report"));
                }
                for i in 0..g.count as usize {
                    let u = usages.get(i).copied().unwrap_or(last);
                    if u >> 16 != 0x59 {
                        return Err(bad("foreign usage page in LampArray"));
                    }
                    entry.1.push(((u & 0xffff) as u16, g.bits));
                }
            }
            _ => {}
        }
        if tag & 0x0c == 0 {
            usages.clear();
            minimum = None;
        }
    }
    if !collections.is_empty() || !saved.is_empty() {
        return Err(bad("unclosed HID scope"));
    }
    for (usage, fields) in expected() {
        if reports.get(&usage).is_none_or(|(_, f)| *f != fields) {
            return Err(DriverError::protocol(format!(
                "non-standard fields of report {usage:02X}"
            )));
        }
    }
    let multi = &reports.get(&0x50).ok_or_else(|| bad("no MultiUpdate"))?.1;
    if multi.len() < 7 || !(multi.len() - 2).is_multiple_of(5) {
        return Err(bad("malformed MultiUpdate"));
    }
    let capacity = (multi.len() - 2) / 5;
    if !(1..=32).contains(&capacity) {
        return Err(bad("unusual MultiUpdate capacity"));
    }
    let mut fields = vec![(3, 8), (0x55, 8)];
    fields.extend(vec![(0x21, 16); capacity]);
    for _ in 0..capacity {
        fields.extend([(0x51, 8), (0x52, 8), (0x53, 8), (0x54, 8)]);
    }
    if *multi != fields {
        return Err(bad("non-standard MultiUpdate"));
    }
    let id = |usage: u16| reports.get(&usage).map_or(0, |(id, _)| *id);
    Ok(ReportMap {
        attributes: id(2),
        request: id(0x20),
        response: id(0x22),
        multi: id(0x50),
        range: id(0x60),
        control: id(0x70),
        capacity,
    })
}
