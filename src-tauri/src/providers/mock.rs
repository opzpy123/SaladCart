use crate::error::{AppError, AppResult};
use crate::providers::image::{ImageRequest, ImageResult};
use ab_glyph::FontVec;
use image::{ImageFormat, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use serde_json::json;
use std::io::Cursor;

pub fn image(request: &ImageRequest) -> AppResult<ImageResult> {
    let (width, height) = image_dims(&request.aspect_ratio);
    let seed = request.seed.unwrap_or(42) as u64;
    let palette = [
        [Rgba([16u8, 20, 16, 255]), Rgba([35, 48, 27, 255])],
        [Rgba([26, 20, 16, 255]), Rgba([82, 41, 28, 255])],
        [Rgba([14, 21, 24, 255]), Rgba([24, 63, 63, 255])],
        [Rgba([22, 18, 27, 255]), Rgba([70, 52, 82, 255])],
    ];
    let pair = palette[(seed as usize) % palette.len()];
    let dark = pair[0];
    let accent = pair[1];
    let mut img = RgbaImage::new(width, height);

    let cx = width as f32 * 0.54;
    let cy = height as f32 * 0.40;
    let glow_r = width as f32 * 0.62;
    for y in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32;
            let ny = y as f32 / height as f32;
            let blend = (nx * 0.55 + ny * 0.45).clamp(0.0, 1.0);
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let glow = (1.0 - ((dx * dx + dy * dy).sqrt() / glow_r)).clamp(0.0, 1.0);
            let glow = glow.powf(2.2) * 0.55;
            let ra = dark.0[0] as f32 + (accent.0[0] as f32 - dark.0[0] as f32) * blend;
            let ga = dark.0[1] as f32 + (accent.0[1] as f32 - dark.0[1] as f32) * blend;
            let ba = dark.0[2] as f32 + (accent.0[2] as f32 - dark.0[2] as f32) * blend;
            let r = (ra + glow * 118.0).min(255.0);
            let g = (ga + glow * 102.0).min(255.0);
            let b = (ba + glow * 52.0).min(255.0);
            let grain = pseudo_random(x as u64, y as u64, seed) * 9.0 - 4.5;
            img.put_pixel(
                x,
                y,
                Rgba([
                    (r + grain).clamp(0.0, 255.0) as u8,
                    (g + grain).clamp(0.0, 255.0) as u8,
                    (b + grain).clamp(0.0, 255.0) as u8,
                    255,
                ]),
            );
        }
    }

    // Abstract portrait silhouette: intentionally graphic so it reads as a
    // production placeholder rather than pretend-generated art.
    let head_cy = height as f32 * 0.38;
    let head_rx = width as f32 * 0.16;
    let head_ry = head_rx * 1.18;
    let shoulder_top = height as f32 * 0.56;
    let shoulder_rx = width as f32 * 0.34;
    let ink = Rgba([15u8, 18, 14, 238]);
    let rim = Rgba([
        (accent.0[0] as f32 * 1.5).min(255.0) as u8,
        (accent.0[1] as f32 * 1.45).min(255.0) as u8,
        (accent.0[2] as f32 * 1.35).min(255.0) as u8,
        255,
    ]);
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - width as f32 * 0.5;
            let dy = y as f32 - head_cy;
            let head = (dx * dx) / (head_rx * head_rx) + (dy * dy) / (head_ry * head_ry);
            let shoulder_dy = y as f32 - shoulder_top;
            let shoulder = (dx * dx) / (shoulder_rx * shoulder_rx)
                + (shoulder_dy * shoulder_dy) / (height as f32 * 0.42).powi(2);
            if y as f32 > shoulder_top && shoulder <= 1.0 {
                img.put_pixel(x, y, ink);
            } else if head <= 1.0 {
                img.put_pixel(x, y, ink);
            }
            if (head > 0.90 && head <= 1.0)
                || (y as f32 > shoulder_top && shoulder > 0.92 && shoulder <= 1.0)
            {
                img.put_pixel(x, y, rim);
            }
        }
    }

    let font = load_font();
    if let Some(font) = &font {
        let pad = (width as f32 * 0.06) as i32;
        let label_scale = ab_glyph::PxScale::from((width as f32 * 0.026).max(18.0));
        draw_text_mut(
            &mut img,
            Rgba([208, 224, 190, 255]),
            pad,
            pad,
            label_scale,
            font,
            "SALADCART  /  MOCK FRAME",
        );
        let panel_h = (height as f32 * 0.22) as u32;
        draw_filled_rect_mut(
            &mut img,
            imageproc::rect::Rect::at(0, (height - panel_h) as i32).of_size(width, panel_h),
            Rgba([12, 14, 11, 196]),
        );
        let title = ascii_title(&request.title);
        let title_scale = ab_glyph::PxScale::from((width as f32 * 0.055).max(26.0));
        draw_text_mut(
            &mut img,
            Rgba([247, 244, 232, 255]),
            pad,
            (height - panel_h + panel_h / 5) as i32,
            title_scale,
            font,
            &title,
        );
        let excerpt = ascii_excerpt(&request.prompt, 64);
        if !excerpt.is_empty() {
            let body_scale = ab_glyph::PxScale::from((width as f32 * 0.024).max(16.0));
            draw_text_mut(
                &mut img,
                Rgba([184, 196, 172, 255]),
                pad,
                (height - panel_h + panel_h / 2) as i32,
                body_scale,
                font,
                &excerpt,
            );
        }
        let foot_scale = ab_glyph::PxScale::from((width as f32 * 0.021).max(14.0));
        draw_text_mut(
            &mut img,
            Rgba([143, 164, 111, 255]),
            pad,
            (height as f32 * 0.86) as i32,
            foot_scale,
            font,
            "LOCAL MOCK · ZERO COST · REPLACE WITH A REAL PROVIDER",
        );
    }

    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|e| AppError::Message(format!("could not encode mock image: {e}")))?;
    Ok(ImageResult {
        bytes,
        mime: "image/png".into(),
        extension: "png".into(),
        cost: 0.0,
        raw: json!({
            "mock": true,
            "width": width,
            "height": height,
            "seed": seed,
            "hasReferences": !request.references.is_empty()
        }),
    })
}

pub struct MockMusic {
    pub bytes: Vec<u8>,
    pub mime: String,
    pub extension: String,
    pub duration_ms: i64,
    pub bpm: f64,
    pub raw: serde_json::Value,
}

pub fn music(prompt: &str, duration_ms: i64, seed: Option<i64>) -> AppResult<MockMusic> {
    let sample_rate = 44_100u32;
    let duration_ms = duration_ms.clamp(8_000, 120_000);
    let total = (sample_rate as i64 * duration_ms / 1000) as usize;
    let bpm = 100.0f64;
    let beat = 60.0 / bpm;
    let mut samples = vec![0.0f32; total];
    let seed = seed.unwrap_or(7) as u64;
    let chords: [[f32; 4]; 4] = [
        [220.00, 261.63, 329.63, 392.00], // Am7
        [174.61, 220.00, 261.63, 329.63], // Fmaj7
        [130.81, 196.00, 261.63, 329.63], // Cmaj7
        [196.00, 246.94, 293.66, 392.00], // G
    ];
    let bars = ((duration_ms as f64 / 1000.0) / (beat * 4.0)).ceil() as usize + 1;
    for bar in 0..bars {
        let chord = chords[(bar + seed as usize) % chords.len()];
        for (n, freq) in chord.iter().enumerate() {
            let start = (bar as f64 * beat * 4.0 * sample_rate as f64) as usize;
            let end = (((bar + 1) as f64 * beat * 4.0) * sample_rate as f64) as usize;
            let end = end.min(total);
            for i in start..end {
                let t = (i - start) as f32 / sample_rate as f32;
                let env = (t / 0.08).min(1.0) * (-(t / 2.4)).exp();
                let wobble = 1.0 + 0.06 * (t * 0.7 + n as f32).sin();
                samples[i] += (2.0 * std::f32::consts::PI * freq * t * wobble).sin() * env * 0.055;
            }
        }
    }
    for beat_index in 0..((duration_ms as f64 / 1000.0) / beat).floor() as usize {
        let start = (beat_index as f64 * beat * sample_rate as f64) as usize;
        let is_kick = beat_index % 4 == 0 || beat_index % 4 == 2;
        if is_kick {
            let len = (sample_rate as f64 * 0.20) as usize;
            for i in 0..len {
                if start + i >= total {
                    break;
                }
                let t = i as f32 / sample_rate as f32;
                let env = (-(t / 0.06)).exp();
                let freq = 105.0 - 55.0 * (t / 0.2).min(1.0);
                samples[start + i] += (2.0 * std::f32::consts::PI * freq * t).sin() * env * 0.42;
            }
        } else {
            let len = (sample_rate as f64 * 0.14) as usize;
            for i in 0..len {
                if start + i >= total {
                    break;
                }
                let t = i as f32 / sample_rate as f32;
                let env = (-(t / 0.035)).exp();
                let noise = pseudo_random(i as u64, beat_index as u64, seed) * 2.0 - 1.0;
                samples[start + i] += noise * env * 0.10;
            }
        }
        let hat_start = start + (beat * 0.5 * sample_rate as f64) as usize;
        let len = (sample_rate as f64 * 0.045) as usize;
        for i in 0..len {
            if hat_start + i >= total {
                break;
            }
            let t = i as f32 / sample_rate as f32;
            let env = (-(t / 0.012)).exp();
            let noise = pseudo_random(i as u64, seed + 3, seed) * 2.0 - 1.0;
            samples[hat_start + i] += noise * env * 0.045;
        }
    }
    // Pentatonic motif, changes every two bars.
    let motif = [440.0, 523.25, 659.25, 587.33, 523.25, 392.0, 440.0, 392.0];
    for (index, freq) in motif.iter().enumerate() {
        let start = ((index as f64 * beat * 1.5) * sample_rate as f64) as usize
            + (seed as usize % 3) * 1000;
        let len = (sample_rate as f64 * beat * 1.25) as usize;
        for i in 0..len {
            if start + i >= total {
                break;
            }
            let t = i as f32 / sample_rate as f32;
            let env = (t / 0.02).min(1.0) * (-(t / 0.45)).exp();
            samples[start + i] += (2.0 * std::f32::consts::PI * freq * t).sin() * env * 0.06;
        }
    }
    let peak = samples
        .iter()
        .fold(0.0f32, |acc, v| acc.max(v.abs()))
        .max(0.001);
    let gain = 0.82 / peak;
    let mut wav = Vec::with_capacity(44 + total * 2);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&((36 + total * 2) as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&((total * 2) as u32).to_le_bytes());
    for sample in &samples {
        let value = (sample * gain * 32767.0).clamp(-32768.0, 32767.0) as i16;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    Ok(MockMusic {
        bytes: wav,
        mime: "audio/wav".into(),
        extension: "wav".into(),
        duration_ms,
        bpm,
        raw: json!({"mock": true, "bpm": bpm, "prompt": prompt, "instrumental": true}),
    })
}

fn image_dims(aspect: &str) -> (u32, u32) {
    match aspect {
        "16:9" => (1600, 900),
        "1:1" => (1200, 1200),
        "4:5" => (1080, 1350),
        _ => (900, 1600),
    }
}

fn load_font() -> Option<FontVec> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    let families = [fontdb::Family::SansSerif];
    let query = fontdb::Query {
        families: &families,
        ..Default::default()
    };
    let id = db.query(&query)?;
    db.with_face_data(id, |data, index| {
        FontVec::try_from_vec_and_index(data.to_vec(), index).ok()
    })
    .flatten()
}

fn ascii_title(value: &str) -> String {
    let ascii: String = value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || "-_".contains(*c))
        .collect();
    let trimmed = ascii.trim();
    if trimmed.is_empty() {
        "PRODUCTION FRAME".into()
    } else {
        trimmed.chars().take(34).collect::<String>().to_uppercase()
    }
}

fn ascii_excerpt(value: &str, max: usize) -> String {
    let ascii: String = value
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect();
    ascii
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max)
        .collect()
}

fn pseudo_random(x: u64, y: u64, seed: u64) -> f32 {
    let mut n = x
        .wrapping_mul(3_744_561)
        .wrapping_add(y.wrapping_mul(6_689_001))
        .wrapping_add(seed.wrapping_mul(2_654_435));
    n ^= n >> 33;
    n = n.wrapping_mul(0xff51afd7ed558ccd);
    n ^= n >> 33;
    (n % 10_000) as f32 / 10_000.0
}
