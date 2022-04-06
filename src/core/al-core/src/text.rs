pub struct LetterTexPosition {
    pub x_min: u32,
    pub x_max: u32,
    pub y_min: u32,
    pub y_max: u32,
    pub x_advance: u32,
    pub y_advance: u32,
    pub w: u32,
    pub h: u32,
    pub bounds: fontdue::OutlineBounds,
}

#[derive(PartialEq)]
struct Letter {
    pub l: char,
    pub w: u32,
    pub h: u32,
    pub x_advance: u32,
    pub y_advance: u32,
    pub bitmap: Vec<u8>,
    pub bounds: fontdue::OutlineBounds,
}
use std::cmp::Ordering;
impl PartialOrd for Letter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let w_cmp = other.w.cmp(&self.w);

        if Ordering::Equal == w_cmp {
            Some(other.h.cmp(&self.h))
        } else {
            Some(w_cmp)
        }
    }
}

use std::collections::HashMap;
pub struct Font {
    pub bitmap: Vec<u8>,
    pub size: Vector2<i32>,
    pub letters: HashMap<char, LetterTexPosition>,
    pub font: fontdue::Font,
}

use cgmath::Vector2;
pub fn rasterize_font(text_size: f32) -> Font {
    // Read the font data.
    let font = include_bytes!("../resources/arial.ttf") as &[u8];
    // Parse it into the font type.
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

    // Rasterize and get the layout metrics for the letter 'g' at 17px.
    let mut h = 0;
    let mut w = 0;

    let mut letters = Vec::new();
    for c in (0 as char)..(255 as char) {
        let (metrics, bitmap) = font.rasterize(c, text_size);

        letters.push(Letter {
            w: metrics.width as u32,
            h: metrics.height as u32,
            x_advance: metrics.advance_width as u32,
            y_advance: metrics.advance_height as u32,
            l: c,
            bounds: metrics.bounds,
            bitmap,
        });

        h += metrics.height;
        w = std::cmp::max(w, metrics.width);
    }

    letters.sort_unstable_by(|l, r| {
        let w_cmp = r.w.cmp(&l.w);

        if Ordering::Equal == w_cmp {
            r.h.cmp(&l.h)
        } else {
            w_cmp
        }
    });

    let h_tex = 512;
    let w_tex = 512;

    let mut letters_tex = HashMap::new();
    let mut x_min = 0;
    let mut y_min = 0;
    let mut size_col = letters[0].w;

    let mut img = vec![0; h_tex * w_tex * 4];

    for Letter {
        l,
        w,
        h,
        x_advance,
        y_advance,
        bitmap,
        bounds,
    } in letters.into_iter()
    {
        let mut i = 0;

        let mut y_max = y_min + h;
        if y_max >= h_tex as u32 {
            y_min = 0;
            y_max = h;
            x_min += size_col;

            size_col = w;
        }

        // Draw here the letter in the tex
        let x_max = x_min + w;
        letters_tex.insert(
            l,
            LetterTexPosition {
                x_min,
                x_max,
                y_min,
                y_max,
                x_advance,
                y_advance,
                bounds,
                w: x_max - x_min,
                h: y_max - y_min,
            },
        );
        for y in (y_min as usize)..(y_max as usize) {
            for x in (x_min as usize)..(x_max as usize) {
                img[4 * (x + w_tex * y)] = bitmap[i];
                img[4 * (x + w_tex * y) + 1] = bitmap[i];
                img[4 * (x + w_tex * y) + 2] = bitmap[i];
                img[4 * (x + w_tex * y) + 3] = bitmap[i];

                i += 1;
            }
        }

        y_min += h;
    }

    Font {
        bitmap: img,
        size: Vector2::new(w_tex as i32, h_tex as i32),
        letters: letters_tex,
        font,
    }
}

mod tests {
    #[test]
    pub fn rasterize_font() {
        use super::Letter;
        use std::cmp::Ordering;

        use super::LetterTexPosition;
        use std::collections::HashMap;
        // Read the font data.
        let font = include_bytes!("../resources/arial.ttf") as &[u8];
        // Parse it into the font type.
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

        // Rasterize and get the layout metrics for the letter 'g' at 17px.
        let mut h = 0;
        let mut w = 0;

        let mut letters = Vec::new();
        for c in 0_u8..255_u8 {
            let (metrics, bitmap) = font.rasterize(c as char, 17.0);

            letters.push(Letter {
                w: metrics.width as u32,
                h: metrics.height as u32,
                x_advance: metrics.advance_width as u32,
                y_advance: metrics.advance_height as u32,
                bounds: metrics.bounds,
                l: c as char,
                bitmap,
            });

            h += metrics.height;
            w = std::cmp::max(w, metrics.width);
        }
        letters.sort_unstable_by(|l, r| {
            let w_cmp = r.w.cmp(&l.w);

            if Ordering::Equal == w_cmp {
                r.h.cmp(&l.h)
            } else {
                w_cmp
            }
        });

        let h_tex = 256 as usize;
        let w_tex = 256 as usize;

        let mut letters_tex = HashMap::new();
        let mut x_min = 0;
        let mut y_min = 0;
        let mut size_col = letters[0].w;

        let mut img = vec![0; (h_tex * w_tex * 4) as usize];

        for Letter {
            l,
            w,
            h,
            x_advance,
            y_advance,
            bitmap,
            bounds,
        } in letters.into_iter()
        {
            let mut i = 0;

            let mut y_max = y_min + h;
            if y_max >= h_tex as u32 {
                y_min = 0;
                y_max = h;
                x_min += size_col;

                size_col = w;
            }

            // Draw here the letter in the tex
            let x_max = x_min + w;
            letters_tex.insert(
                l,
                LetterTexPosition {
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                    x_advance,
                    y_advance,
                    w: x_max - x_min,
                    h: y_max - y_min,
                    bounds,
                },
            );
            for y in (y_min as usize)..(y_max as usize) {
                for x in (x_min as usize)..(x_max as usize) {
                    img[4 * (x + w_tex * y)] = bitmap[i];
                    img[4 * (x + w_tex * y) + 1] = bitmap[i];
                    img[4 * (x + w_tex * y) + 2] = bitmap[i];
                    img[4 * (x + w_tex * y) + 3] = bitmap[i];

                    i += 1;
                }
            }

            y_min += h;
        }
    }
}