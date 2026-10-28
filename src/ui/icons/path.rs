//! The SVG path subset the icon set uses (absolute M, L, H, V, C, Z), flattened to polylines in
//! the 24 x 24 icon grid. Parsed once per glyph. Free of egui: `build.rs` includes this file.

/// A point in the 24 x 24 icon grid. Plain data, so the build script can share this file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub const fn pos2(x: f32, y: f32) -> Point {
    Point { x, y }
}

/// Segments per cubic curve: smooth at 24 px, cheap to tessellate.
const CURVE_STEPS: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub struct Polyline {
    pub points: Vec<Point>,
    pub closed: bool,
}

fn numbers(text: &str) -> Vec<f32> {
    let mut out = Vec::new();
    let mut current = String::new();
    let flush = |current: &mut String, out: &mut Vec<f32>| {
        if let Ok(v) = current.parse() {
            out.push(v);
        }
        current.clear();
    };
    for ch in text.chars() {
        match ch {
            '0'..='9' | '.' => current.push(ch),
            '-' => {
                flush(&mut current, &mut out);
                current.push(ch);
            }
            _ => flush(&mut current, &mut out),
        }
    }
    flush(&mut current, &mut out);
    out
}

fn cubic(p0: Point, p1: Point, p2: Point, p3: Point, out: &mut Vec<Point>) {
    for step in 1..=CURVE_STEPS {
        let t = step as f32 / CURVE_STEPS as f32;
        let u = 1.0 - t;
        let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
        out.push(pos2(
            w[0] * p0.x + w[1] * p1.x + w[2] * p2.x + w[3] * p3.x,
            w[0] * p0.y + w[1] * p1.y + w[2] * p2.y + w[3] * p3.y,
        ));
    }
}

/// Unknown commands end the parse: the glyph table is generated, and a test checks every glyph.
pub fn parse(d: &str) -> Option<Vec<Polyline>> {
    let mut lines = Vec::new();
    let mut current: Vec<Point> = Vec::new();
    let mut at = pos2(0.0, 0.0);
    let starts: Vec<usize> = d
        .char_indices()
        .filter(|(_, c)| c.is_ascii_alphabetic())
        .map(|(i, _)| i)
        .collect();
    for (n, &start) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(d.len());
        let command = d[start..].chars().next()?;
        let args = numbers(&d[start + 1..end]);
        match command {
            'M' => {
                if current.len() > 1 {
                    lines.push(Polyline {
                        points: std::mem::take(&mut current),
                        closed: false,
                    });
                }
                current.clear();
                // Pairs after the first are implicit line-tos.
                for p in args.as_chunks::<2>().0 {
                    at = pos2(p[0], p[1]);
                    current.push(at);
                }
            }
            'L' => {
                for p in args.as_chunks::<2>().0 {
                    at = pos2(p[0], p[1]);
                    current.push(at);
                }
            }
            'H' => {
                for x in &args {
                    at = pos2(*x, at.y);
                    current.push(at);
                }
            }
            'V' => {
                for y in &args {
                    at = pos2(at.x, *y);
                    current.push(at);
                }
            }
            'C' => {
                for p in args.as_chunks::<6>().0 {
                    let end = pos2(p[4], p[5]);
                    cubic(at, pos2(p[0], p[1]), pos2(p[2], p[3]), end, &mut current);
                    at = end;
                }
            }
            'Z' => {
                if current.len() > 1 {
                    lines.push(Polyline {
                        points: std::mem::take(&mut current),
                        closed: true,
                    });
                }
            }
            _ => return None,
        }
    }
    if current.len() > 1 {
        lines.push(Polyline {
            points: current,
            closed: false,
        });
    }
    Some(lines)
}

/// Every element of a glyph as polylines. Unparseable paths are dropped; a test checks the set.
pub fn outline(elements: &[super::glyphs::Element]) -> Vec<Polyline> {
    elements
        .iter()
        .flat_map(|e| match e {
            super::glyphs::Element::Path(d) => parse(d).unwrap_or_default(),
            super::glyphs::Element::Circle { cx, cy, r } => vec![circle(*cx, *cy, *r)],
        })
        .collect()
}

pub fn circle(cx: f32, cy: f32, r: f32) -> Polyline {
    const STEPS: usize = 32;
    let points = (0..STEPS)
        .map(|i| {
            let a = i as f32 / STEPS as f32 * std::f32::consts::TAU;
            pos2(cx + r * a.cos(), cy + r * a.sin())
        })
        .collect();
    Polyline {
        points,
        closed: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lines_curves_and_close() {
        let lines = parse("M4 5L20 5M4 12H20V14Z").unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].points, vec![pos2(4.0, 5.0), pos2(20.0, 5.0)]);
        assert!(lines[1].closed);
        assert_eq!(lines[1].points.last(), Some(&pos2(20.0, 14.0)));
        let curve = parse("M0 0C0 10 10 10 10 0").unwrap();
        assert_eq!(curve[0].points.len(), 1 + CURVE_STEPS);
        assert_eq!(curve[0].points.last(), Some(&pos2(10.0, 0.0)));
        assert!(parse("M0 0A1 1 0 0 1 2 2").is_none());
    }

    #[test]
    fn negative_numbers_without_separators() {
        assert_eq!(numbers("1.5-2 3"), vec![1.5, -2.0, 3.0]);
    }
}
