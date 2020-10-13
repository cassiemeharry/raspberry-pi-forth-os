use core::fmt;
use font8x8;
use spin::Mutex;

use crate::rpi::framebuffer::{Framebuffer, Pixel};

#[derive(Clone, Debug)]
struct CellOffset {
    offset_x_cells: u32,
    offset_y_cells: u32,
    screen_width_cells: u32,
    screen_height_cells: u32,
}

impl CellOffset {
    #[inline]
    fn update_from_fb(&mut self, fb: &Framebuffer) {
        self.screen_width_cells = fb.width() / 8;
        self.screen_height_cells = fb.height() / 8;
    }

    #[inline]
    fn advance(&mut self) {
        self.offset_x_cells += 1;
        if self.offset_x_cells >= self.screen_width_cells {
            self.offset_x_cells = 0;
            self.offset_y_cells += 1;
        }
    }

    // #[inline]
    // fn set_cell_xy(&mut self, x: u32, y: u32) {
    //     debug_assert!(x >= 0);
    //     debug_assert!(x < self.screen_width_cells);
    //     debug_assert!(y >= 0);
    //     // Note: this looks like an off-by-one error, but we do want the
    //     // `offset_cells` to be able to index within the first row past the
    //     // screen. This is checked for by `Self::needs_scroll`.
    //     debug_assert!(y <= self.screen_height_cells);
    //     self.offset_cells = (self.screen_width_cells * y) + x;
    // }

    #[inline]
    fn cell_xy(&self) -> (u32, u32) {
        (self.offset_x_cells, self.offset_y_cells)
        // let y = self.offset_cells / self.screen_width_cells;
        // let x = self.offset_cells % self.screen_width_cells;
        // (x, y)
    }

    #[inline]
    fn top_left_pixel_xy(&self) -> (u32, u32) {
        (self.offset_x_cells * 8, self.offset_y_cells * 8)
    }

    #[inline]
    fn move_to_line_start(&mut self) {
        self.offset_x_cells = 0;
    }

    #[inline]
    fn move_to_next_line(&mut self) {
        self.offset_y_cells += 1;
    }

    #[inline]
    fn needs_scroll(&self) -> bool {
        self.offset_y_cells >= self.screen_height_cells
    }

    fn scroll_down(&mut self, fb: &mut Framebuffer, lines: u32) {
        // println!(
        //     "Scrolling down {} line{}",
        //     lines,
        //     if lines == 1 { "" } else { "s" }
        // );

        // Copy the pixels after the divider backwards.
        let offset = (fb.width() as usize) * lines as usize * 8;
        let pixels = fb.pixels_mut();
        pixels.copy_within(offset.., 0);
        // Override the copied from and not overridden pixels with black.
        let start_pixel_offset = pixels.len() - offset;
        for pixel in pixels[start_pixel_offset..].iter_mut() {
            *pixel = Pixel::from((0, 0, 0));
        }
        // Update the cell offset to match
        self.offset_y_cells -= 1;
    }
}

static CELL_OFFSET: Mutex<CellOffset> = Mutex::new(CellOffset {
    offset_x_cells: 0,
    offset_y_cells: 0,
    screen_width_cells: 640 / 8,
    screen_height_cells: 480 / 8,
});

fn lookup_codepoint(c: u32) -> Option<[u8; 8]> {
    macro_rules! check_block {
        ($range:expr, $block:expr) => {
            let range = $range;
            if range.contains(&c) {
                let block_offset = c - *range.start();
                return Some($block[block_offset as usize].byte_array());
            }
        };
    }

    check_block!(0x0000..=0x007f, font8x8::unicode::BASIC_UNICODE);
    check_block!(0x2580..=0x259f, font8x8::unicode::BLOCK_UNICODE);
    check_block!(0x2500..=0x257f, font8x8::unicode::BOX_UNICODE);
    check_block!(0x0390..=0x03c9, font8x8::unicode::GREEK_UNICODE);
    check_block!(0x3040..=0x309f, font8x8::unicode::HIRAGANA_UNICODE);
    check_block!(0x00A0..=0x00FF, font8x8::unicode::LATIN_UNICODE);
    check_block!(0x20a7..=0x20a7, font8x8::unicode::MISC_UNICODE[0..=0]);
    check_block!(0x0192..=0x0192, font8x8::unicode::MISC_UNICODE[1..=1]);
    check_block!(0x00aa..=0x00aa, font8x8::unicode::MISC_UNICODE[2..=2]);
    check_block!(0x00ba..=0x00ba, font8x8::unicode::MISC_UNICODE[3..=3]);
    check_block!(0x2310..=0x2310, font8x8::unicode::MISC_UNICODE[4..=4]);
    check_block!(0x2264..=0x2264, font8x8::unicode::MISC_UNICODE[5..=5]);
    check_block!(0x2265..=0x2265, font8x8::unicode::MISC_UNICODE[6..=6]);
    check_block!(0x0060..=0x0060, font8x8::unicode::MISC_UNICODE[7..=7]);
    check_block!(0x1ef2..=0x1ef2, font8x8::unicode::MISC_UNICODE[8..=8]);
    check_block!(0x1ef3..=0x1ef3, font8x8::unicode::MISC_UNICODE[9..=9]);
    check_block!(0xe543..=0xe55a, font8x8::unicode::SGA_UNICODE);

    None
}

pub struct Console {}

impl Console {
    pub fn new() -> Option<Console> {
        if !Framebuffer::ready() {
            return None;
        }
        Some(Console {})
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            write_char(c);
        }
        Ok(())
    }
}

fn draw_char(fb: &mut Framebuffer, pixel_x: u32, pixel_y: u32, bytes: [u8; 8]) {
    let black = Pixel::from((0x00, 0x00, 0x00));
    let white = Pixel::from((0xFF, 0xFF, 0xFF));
    for (y_offset, byte) in bytes.iter().enumerate() {
        let y_offset = y_offset as u32;
        let y = pixel_y + y_offset;
        let byte = *byte;
        for x_offset in 0..8 {
            let x = pixel_x + x_offset;
            let bit_on = (byte & (1 << x_offset)) != 0;
            if bit_on {
                fb[(x, y)] = white;
            } else {
                fb[(x, y)] = black;
            }
        }
    }
}

pub fn write_char(c: char) {
    Framebuffer::with(|fb| {
        let mut cell_offset = CELL_OFFSET.lock();
        {
            while cell_offset.needs_scroll() {
                cell_offset.scroll_down(fb, 1);
            }
            const EMPTY: [u8; 8] = [0; 8];
            // println!(
            //     "Clearing cell at {:?} (pixel coords: {:?})",
            //     cell_offset.cell_xy(),
            //     cell_offset.top_left_pixel_xy()
            // );
            let (x, y) = cell_offset.top_left_pixel_xy();
            draw_char(fb, x, y, EMPTY);
        }
        if c == '\r' {
            cell_offset.move_to_line_start();
        } else if c == '\n' {
            cell_offset.move_to_line_start();
            cell_offset.move_to_next_line();
            return;
        } else {
            let codepoint = c as u32;
            let bytes: [u8; 8] = match lookup_codepoint(codepoint) {
                Some(r) => r,
                None => [0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00],
            };
            cell_offset.update_from_fb(fb);

            let (cell_x, cell_y) = cell_offset.cell_xy();
            let (x_base, y_base) = cell_offset.top_left_pixel_xy();

            // println!(
            //     "Drawing character {:?} with {:?} at {:?} ({:?})",
            //     c,
            //     bytes,
            //     (cell_x, cell_y),
            //     &*cell_offset
            // );

            draw_char(fb, x_base, y_base, bytes);
            cell_offset.advance();
        }

        while cell_offset.needs_scroll() {
            cell_offset.scroll_down(fb, 1);
        }
        {
            const BLOCK: [u8; 8] = [0xFF; 8];
            let (x, y) = cell_offset.top_left_pixel_xy();
            draw_char(fb, x, y, BLOCK);
        }
    });
}
