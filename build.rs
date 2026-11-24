use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

fn render_pin_window_rgba(size: u32) -> Vec<u8> {
    let w = size as i32;
    let h = size as i32;
    let mut pixels = vec![0u8; (w * h * 4) as usize];

    let set_px = |buf: &mut [u8], x: i32, y: i32, r: u8, g: u8, b: u8, a: u8| {
        if x >= 0 && x < w && y >= 0 && y < h {
            let idx = ((y * w + x) * 4) as usize;
            buf[idx] = r;
            buf[idx + 1] = g;
            buf[idx + 2] = b;
            buf[idx + 3] = a;
        }
    };

    let s = |v: f32| -> i32 { (v * size as f32 / 32.0).round() as i32 };
    let sf = |v: f32| -> f32 { v * size as f32 / 32.0 };

    let win_x0 = s(4.0);
    let win_x1 = s(24.0);
    let win_y0 = s(10.0);
    let win_y1 = s(27.0);
    let title_y1 = s(14.0);
    let border_t = (size as f32 / 32.0).max(1.0).round() as i32;

    for y in win_y0..=win_y1 {
        for x in win_x0..=win_x1 {
            let is_border = (x - win_x0).abs() < border_t
                || (win_x1 - x).abs() < border_t
                || (y - win_y0).abs() < border_t
                || (win_y1 - y).abs() < border_t;
            let is_titlebar = y >= win_y0 + border_t
                && y <= title_y1
                && x >= win_x0 + border_t
                && x <= win_x1 - border_t;

            if is_border {
                set_px(&mut pixels, x, y, 30, 86, 160, 255);
            } else if is_titlebar {
                set_px(&mut pixels, x, y, 52, 125, 215, 255);
            } else {
                set_px(&mut pixels, x, y, 242, 246, 252, 255);
            }
        }
    }

    let line_x0 = s(7.0);
    let line_x1 = s(16.0);
    let line1_y = s(18.0);
    let line2_y = s(22.0);
    for y_off in 0..border_t {
        for x in line_x0..=line_x1 {
            set_px(&mut pixels, x, line1_y + y_off, 160, 175, 195, 255);
            set_px(&mut pixels, x, line2_y + y_off, 160, 175, 195, 255);
        }
    }

    let steps = (sf(8.0).round() as i32).max(1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let px = (sf(19.0) - t * sf(8.0)).round() as i32;
        let py = (sf(12.0) + t * sf(8.0)).round() as i32;
        for off in 0..border_t {
            set_px(&mut pixels, px + off, py, 95, 105, 120, 255);
            set_px(&mut pixels, px + off + 1, py, 130, 140, 155, 255);
        }
    }

    let cx = sf(22.0);
    let cy = sf(8.0);
    let r_inner = sf(4.6);
    let r_outer = sf(5.4);
    let box_x0 = (cx - r_outer - 1.0).floor() as i32;
    let box_x1 = (cx + r_outer + 1.0).ceil() as i32;
    let box_y0 = (cy - r_outer - 1.0).floor() as i32;
    let box_y1 = (cy + r_outer + 1.0).ceil() as i32;

    for y in box_y0..=box_y1 {
        for x in box_x0..=box_x1 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= r_inner {
                if dx < -0.5 * sf(1.0) && dy < -0.5 * sf(1.0) {
                    set_px(&mut pixels, x, y, 255, 140, 140, 255);
                } else {
                    set_px(&mut pixels, x, y, 215, 45, 55, 255);
                }
            } else if dist <= r_outer {
                set_px(&mut pixels, x, y, 125, 20, 28, 255);
            }
        }
    }

    let base_span = (sf(2.0).round() as i32).max(1);
    for step in -base_span..=base_span {
        let t = step as f32 / base_span as f32;
        let px = (sf(19.0) + t * sf(2.0)).round() as i32;
        let py = (sf(11.0) - t * sf(2.0)).round() as i32;
        for off in 0..border_t {
            set_px(&mut pixels, px + off, py, 200, 40, 50, 255);
            set_px(&mut pixels, px + off + 1, py + 1, 140, 25, 35, 255);
        }
    }

    pixels
}

fn write_ico_file(path: &Path, sizes: &[u32]) -> io::Result<()> {
    let mut entries_rgba: Vec<(u32, Vec<u8>)> = Vec::with_capacity(sizes.len());
    for &sz in sizes {
        entries_rgba.push((sz, render_pin_window_rgba(sz)));
    }

    let num_images = entries_rgba.len() as u16;
    let header_size = 6usize;
    let entry_size = 16usize;
    let mut data_offset = (header_size + entry_size * entries_rgba.len()) as u32;

    struct DibImage {
        size: u32,
        dib_bytes: Vec<u8>,
        offset: u32,
    }

    let mut dibs: Vec<DibImage> = Vec::with_capacity(entries_rgba.len());

    for (sz, rgba) in entries_rgba {
        let w = sz as i32;
        let h = sz as i32;
        let xor_size = (sz * sz * 4) as usize;
        let and_row_bytes = (((sz + 31) / 32) * 4) as usize;
        let and_size = and_row_bytes * (sz as usize);
        let bih_size = 40u32;

        let mut dib = Vec::with_capacity(bih_size as usize + xor_size + and_size);

        dib.extend_from_slice(&bih_size.to_le_bytes());
        dib.extend_from_slice(&w.to_le_bytes());
        dib.extend_from_slice(&(h * 2).to_le_bytes());
        dib.extend_from_slice(&1u16.to_le_bytes());
        dib.extend_from_slice(&32u16.to_le_bytes());
        dib.extend_from_slice(&0u32.to_le_bytes());
        dib.extend_from_slice(&(xor_size as u32).to_le_bytes());
        dib.extend_from_slice(&0i32.to_le_bytes());
        dib.extend_from_slice(&0i32.to_le_bytes());
        dib.extend_from_slice(&0u32.to_le_bytes());
        dib.extend_from_slice(&0u32.to_le_bytes());

        for y in 0..sz {
            let src_y = sz - 1 - y;
            for x in 0..sz {
                let idx = ((src_y * sz + x) * 4) as usize;
                let r = rgba[idx];
                let g = rgba[idx + 1];
                let b = rgba[idx + 2];
                let a = rgba[idx + 3];
                dib.push(b);
                dib.push(g);
                dib.push(r);
                dib.push(a);
            }
        }

        for y in 0..sz {
            let src_y = sz - 1 - y;
            let mut row = vec![0u8; and_row_bytes];
            for x in 0..sz {
                let alpha = rgba[((src_y * sz + x) * 4 + 3) as usize];
                if alpha == 0 {
                    row[(x / 8) as usize] |= 1 << (7 - (x % 8));
                }
            }
            dib.extend_from_slice(&row);
        }

        let len = dib.len() as u32;
        dibs.push(DibImage {
            size: sz,
            dib_bytes: dib,
            offset: data_offset,
        });
        data_offset += len;
    }

    let mut file = File::create(path)?;

    file.write_all(&0u16.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&num_images.to_le_bytes())?;

    for img in &dibs {
        let w_byte: u8 = if img.size >= 256 { 0 } else { img.size as u8 };
        let h_byte: u8 = if img.size >= 256 { 0 } else { img.size as u8 };
        file.write_all(&[w_byte, h_byte, 0, 0])?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&32u16.to_le_bytes())?;
        file.write_all(&(img.dib_bytes.len() as u32).to_le_bytes())?;
        file.write_all(&img.offset.to_le_bytes())?;
    }

    for img in &dibs {
        file.write_all(&img.dib_bytes)?;
    }

    file.flush()?;
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    {
        let assets_dir = Path::new("assets");
        let _ = fs::create_dir_all(assets_dir);
        let ico_path = assets_dir.join("pin_window.ico");

        if let Err(e) = write_ico_file(&ico_path, &[16, 32, 256]) {
            panic!("Failed to generate pin_window.ico: {}", e);
        }

        let mut res = winres::WindowsResource::new();
        res.set_icon(ico_path.to_str().expect("Invalid UTF-8 path"));
        if let Err(e) = res.compile() {
            panic!("Failed to compile Windows resources: {}", e);
        }
    }
}
