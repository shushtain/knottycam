use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Arc, Line, PrimitiveStyle},
};
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use std::error::Error;
use std::time::{Duration, Instant};
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, Format, FourCC};

const CAMERA_RESOLUTION_X: usize = 160;
const CAMERA_RESOLUTION_Y: usize = 120;
const CELL_COUNT: usize = 1080 / 18;
const CELL_SIZE: usize = 18;
const PIXELS_IN_CELL: usize = CAMERA_RESOLUTION_Y / CELL_COUNT;
const OUTPUT_RESOLUTION: usize = CELL_COUNT * CELL_SIZE;

struct LayerStyle {
    color: Rgb888,
    thickness: u32,
}

const LAYERS: [LayerStyle; 4] = [
    LayerStyle {
        color: Rgb888::new(115, 38, 77),
        // thickness: CELL_SIZE as u32 / 2,
        thickness: 8,
    },
    LayerStyle {
        color: Rgb888::new(191, 64, 64),
        // thickness: CELL_SIZE as u32 / 3,
        thickness: 6,
    },
    LayerStyle {
        color: Rgb888::new(236, 151, 122),
        // thickness: CELL_SIZE as u32 / 4,
        thickness: 4,
    },
    LayerStyle {
        color: Rgb888::new(255, 230, 205),
        // thickness: CELL_SIZE as u32 / 5,
        thickness: 2,
    },
];

struct MinifbDrawTarget<'a> {
    buffer: &'a mut [u32],
    width: usize,
    height: usize,
}

impl<'a> DrawTarget for MinifbDrawTarget<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                let index = (y as usize * self.width) + x as usize;
                self.buffer[index] = 0xFF000000
                    | (color.r() as u32) << 16
                    | (color.g() as u32) << 8
                    | (color.b() as u32);
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for MinifbDrawTarget<'a> {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let device = Device::new(0)?;
    let format = device.set_format(&Format::new(
        CAMERA_RESOLUTION_X as u32,
        CAMERA_RESOLUTION_Y as u32,
        FourCC::new(b"YUYV"),
    ))?;
    let mut stream = Stream::with_buffers(&device, Type::VideoCapture, 4)?;
    let mut window = Window::new(
        "Knotty Cam",
        OUTPUT_RESOLUTION,
        OUTPUT_RESOLUTION,
        WindowOptions::default(),
    )?;

    let mut cell_buffer: Vec<[u8; 4]> = vec![[0; 4]; CELL_COUNT * CELL_COUNT];
    let mut pixel_buffer: Vec<u32> = vec![0; OUTPUT_RESOLUTION * OUTPUT_RESOLUTION];

    let mut shapes: Vec<Vec<Vec<u32>>> =
        vec![vec![vec![0; CELL_SIZE * CELL_SIZE]; 16]; LAYERS.len()];
    let mut shape_buffer: Vec<u32> = vec![0; CELL_SIZE * CELL_SIZE];
    let p_start = 0;
    let p_center = CELL_SIZE as i32 / 2;
    let p_end = CELL_SIZE as i32;
    let c_start = CELL_SIZE as i32 / -2;
    let c_end = CELL_SIZE as i32 / 2;
    let mut fills: Vec<PrimitiveStyle<Rgb888>> = Vec::with_capacity(LAYERS.len());
    let mut strokes: Vec<PrimitiveStyle<Rgb888>> = Vec::with_capacity(LAYERS.len());

    for layer in 0..shapes.len() {
        fills.push(PrimitiveStyle::with_fill(LAYERS[layer].color));
        strokes.push(PrimitiveStyle::with_stroke(
            LAYERS[layer].color,
            LAYERS[layer].thickness,
        ));
    }

    for layer in 0..shapes.len() {
        for sum in 0..16 {
            shape_buffer.fill(0);
            let mut shape = MinifbDrawTarget {
                buffer: &mut shape_buffer[..],
                width: CELL_SIZE,
                height: CELL_SIZE,
            };
            match sum {
                0 => {
                    // Circle::new(Point::new(p_center, p_center), LAYERS[layer].thickness)
                    //     .into_styled(fills[layer])
                    //     .draw(&mut shape)?;
                }
                1 => {
                    Line::new(
                        Point::new(p_center, p_start),
                        Point::new(p_center, p_center),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                }
                2 => {
                    Line::new(Point::new(p_center, p_center), Point::new(p_end, p_center))
                        .into_styled(strokes[layer])
                        .draw(&mut shape)?;
                }
                3 => Arc::new(
                    Point::new(c_end, c_start),
                    CELL_SIZE as u32,
                    90.0.deg(),
                    90.0.deg(),
                )
                .into_styled(strokes[layer])
                .draw(&mut shape)?,

                4 => Line::new(Point::new(p_center, p_center), Point::new(p_center, p_end))
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?,
                5 => Line::new(Point::new(p_center, p_start), Point::new(p_center, p_end))
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?,
                6 => {
                    Arc::new(
                        Point::new(c_end, c_end),
                        CELL_SIZE as u32,
                        180.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                }
                7 => {
                    Arc::new(
                        Point::new(c_end, c_start),
                        CELL_SIZE as u32,
                        90.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                    Arc::new(
                        Point::new(c_end, c_end),
                        CELL_SIZE as u32,
                        180.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?
                }
                8 => Line::new(
                    Point::new(p_start, p_center),
                    Point::new(p_center, p_center),
                )
                .into_styled(strokes[layer])
                .draw(&mut shape)?,
                9 => {
                    Arc::new(
                        Point::new(c_start, c_start),
                        CELL_SIZE as u32,
                        0.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                }
                10 => Line::new(Point::new(p_start, p_center), Point::new(p_end, p_center))
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?,
                11 => {
                    Arc::new(
                        Point::new(c_end, c_start),
                        CELL_SIZE as u32,
                        90.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                    Arc::new(
                        Point::new(c_start, c_start),
                        CELL_SIZE as u32,
                        0.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?
                }
                12 => {
                    Arc::new(
                        Point::new(c_start, c_end),
                        CELL_SIZE as u32,
                        270.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                }
                13 => {
                    Arc::new(
                        Point::new(c_start, c_start),
                        CELL_SIZE as u32,
                        0.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                    Arc::new(
                        Point::new(c_start, c_end),
                        CELL_SIZE as u32,
                        270.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?
                }
                14 => {
                    Arc::new(
                        Point::new(c_end, c_end),
                        CELL_SIZE as u32,
                        180.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                    Arc::new(
                        Point::new(c_start, c_end),
                        CELL_SIZE as u32,
                        270.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?
                }
                15 => {
                    Arc::new(
                        Point::new(c_start, c_start),
                        CELL_SIZE as u32,
                        0.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                    Arc::new(
                        Point::new(c_end, c_start),
                        CELL_SIZE as u32,
                        90.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                    Arc::new(
                        Point::new(c_end, c_end),
                        CELL_SIZE as u32,
                        180.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?;
                    Arc::new(
                        Point::new(c_start, c_end),
                        CELL_SIZE as u32,
                        270.0.deg(),
                        90.0.deg(),
                    )
                    .into_styled(strokes[layer])
                    .draw(&mut shape)?
                }
                _ => (),
            }
            shapes[layer][sum].copy_from_slice(&shape_buffer);
        }
    }

    let bleed: usize = (CAMERA_RESOLUTION_X - CAMERA_RESOLUTION_Y) / 2;

    let mut last_frame_time = Instant::now();
    let target_frame_duration = Duration::from_secs(1 / 8);
    let mut frame_parity = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let elapsed_time = now.duration_since(last_frame_time);

        if elapsed_time < target_frame_duration {
            continue;
        }
        last_frame_time = Instant::now();
        frame_parity = (frame_parity + 1) % 2;

        // cell_buffer.fill([0; 4]);
        let (frame_data, _meta) = stream.next()?;

        let input_width = format.width as usize;
        let input_height = format.height as usize;
        let stride = format.stride as usize;

        cell_buffer
            .par_iter_mut()
            .enumerate()
            .for_each(|(cell_index, cell_data_ref)| {
                let cell_y = cell_index / CELL_COUNT;
                let cell_x_raw = cell_index % CELL_COUNT;
                let cell_x = CELL_COUNT - 1 - cell_x_raw;
                if cell_x == 0
                    || cell_x == CELL_COUNT - 1
                    || cell_y == 0
                    || cell_y == CELL_COUNT - 1
                {
                    return;
                }

                if (cell_x + cell_y) % 2 != frame_parity {
                    return;
                }
                cell_data_ref.fill(0);

                let mut total_luminance: u32 = 0;
                let mut pixel_count: u32 = 0;

                let source_block_x = bleed + (cell_x * PIXELS_IN_CELL);
                let source_block_y = cell_y * PIXELS_IN_CELL;

                for block_y in 0..PIXELS_IN_CELL {
                    for block_x in 0..PIXELS_IN_CELL {
                        let source_y = source_block_y + block_y;
                        let source_x = source_block_x + block_x;

                        if source_x < input_width && source_y < input_height {
                            let yuyv_block = source_y * stride + (source_x / 2) * 4;
                            let byte_offset = if source_x % 2 == 0 { 0 } else { 2 };

                            let luminance_index = yuyv_block + byte_offset;
                            let luminance = if luminance_index < frame_data.len() {
                                frame_data[luminance_index] as u32
                            } else {
                                255
                            };
                            total_luminance += luminance;
                            pixel_count += 1;
                        }
                    }
                }

                let average_luminance = if pixel_count > 0 {
                    total_luminance / pixel_count
                    // total_luminance / pixel_count * 100 / 255
                } else {
                    0
                };

                if average_luminance >= 20 && average_luminance <= 50 {
                    cell_data_ref[0] = 1;
                }
                if average_luminance >= 40 && average_luminance <= 70 {
                    cell_data_ref[1] = 1;
                }
                if average_luminance >= 60 && average_luminance <= 90 {
                    cell_data_ref[2] = 1;
                }
                if average_luminance >= 80 && average_luminance <= 100 {
                    cell_data_ref[3] = 1;
                }
            });

        pixel_buffer
            .par_iter_mut()
            .for_each(|p| *p = 0xFF000000 | (32 << 16) | (19 << 8) | 32);

        pixel_buffer
            .par_chunks_exact_mut(OUTPUT_RESOLUTION)
            .enumerate()
            .for_each(|(row_index, row_slice)| {
                let y = row_index / CELL_SIZE;
                for x in 0..CELL_COUNT {
                    let i0 = y * CELL_COUNT + x;

                    if cell_buffer[i0] == [0; 4] {
                        continue;
                    }

                    let i1 = (y - 1) * CELL_COUNT + x;
                    let i2 = y * CELL_COUNT + x + 1;
                    let i4 = (y + 1) * CELL_COUNT + x;
                    let i8 = y * CELL_COUNT + x - 1;

                    for k in 0..4 {
                        if cell_buffer[i0][k] == 0 {
                            continue;
                        }

                        let sum = cell_buffer[i1][k] * 1
                            + cell_buffer[i2][k] * 2
                            + cell_buffer[i4][k] * 4
                            + cell_buffer[i8][k] * 8;

                        let dest_x = x * CELL_SIZE;
                        let py = row_index % CELL_SIZE;

                        for px in 0..CELL_SIZE {
                            let shape_pixel = shapes[k][sum as usize][py * CELL_SIZE + px];
                            if shape_pixel != 0 {
                                let target_index = dest_x + px;
                                row_slice[target_index] = shape_pixel;
                            }
                        }
                    }
                }
            });

        window.update_with_buffer(&pixel_buffer, OUTPUT_RESOLUTION, OUTPUT_RESOLUTION)?;
    }

    Ok(())
}
