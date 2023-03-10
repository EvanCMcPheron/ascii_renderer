use super::char_buffer::CharBuffer;
use super::Vector2;

/// The struct fed to a CharBuffer for drawing lines.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Line {
    pub char: char,
    pub points: (Vector2, Vector2),
}

impl Into<((usize, usize), (usize, usize))> for Line {
    fn into(self) -> ((usize, usize), (usize, usize)) {
        #[inline]
        fn f32_to_usize(value: f32) -> usize {
            value.round() as usize
        }

        if (self.points.0.x < 0.0 && self.points.1.x < 0.0)
            || (self.points.0.y < 0.0 && self.points.1.y < 0.0)
        {
            //line is entirely offscreen, the coords are set near the 32-bit unsigned int limit
            return (
                (4_000_000_000, 4_000_000_000),
                (4_000_000_000, 4_000_000_000),
            );
        }
        if self.points.0.x == self.points.1.x || self.points.0.y == self.points.1.y {
            //Vertical lines and Horizontal lines - converting direcly to usize is valid as it will just shift any offscreen endpoint vertically/horizontally until they are 0
            return (
                (f32_to_usize(self.points.0.x), f32_to_usize(self.points.0.y)),
                (f32_to_usize(self.points.1.x), f32_to_usize(self.points.1.y)),
            );
        }

        let first = if self.points.0.x < self.points.1.x {
            self.points.0
        } else {
            self.points.1
        }; //Handles if the second point comes before the first.
        let second = if self.points.0.x < self.points.1.x {
            self.points.1
        } else {
            self.points.0
        };

        let slope = (second.y - first.y) / (second.x - first.x);
        let equation_for_y = |x: f32| slope * (x - first.x) + first.y;
        let equation_for_x = |y: f32| (y - first.y) / slope + first.x;

        if first.x < 0.0 {
            //If the first point is offscreen to the left
            let ret = ((0.0, equation_for_y(0.0)), (second.x, second.y));
            if ret.0 .1 < 0.0 {
                return (
                    (f32_to_usize(equation_for_x(0.0)), 0),
                    (f32_to_usize(ret.1 .0), f32_to_usize(ret.1 .1)),
                );
            }
            return (
                (f32_to_usize(ret.0 .0), f32_to_usize(ret.0 .1)),
                (f32_to_usize(ret.1 .0), f32_to_usize(ret.1 .1)),
            );
        }
        if first.y < 0.0 {
            //If the first point is above the screen
            let ret = ((equation_for_x(0.0), 0.0), (second.x, second.y));
            if ret.0 .0 < 0.0 {
                return {
                    (
                        (0, f32_to_usize(equation_for_y(0.0))),
                        (f32_to_usize(ret.1 .0), f32_to_usize(ret.1 .1)),
                    )
                };
            }
            return (
                (f32_to_usize(ret.0 .0), f32_to_usize(ret.0 .1)),
                (f32_to_usize(ret.1 .0), f32_to_usize(ret.1 .1)),
            );
        } else if second.y < 0.0 {
            //If the second point is above the screen
            return (
                (f32_to_usize(first.x), f32_to_usize(first.y)),
                (f32_to_usize(equation_for_x(0.0)), 0),
            );
        } // We don't need to worry about if the second point is left of the screen, as we know that the first point is the leftmost one and if they are both offscreen to the left then there was an early return

        (
            (f32_to_usize(self.points.0.x), f32_to_usize(self.points.0.y)),
            (f32_to_usize(self.points.1.x), f32_to_usize(self.points.1.y)),
        )
    }
}

impl CharBuffer {
    pub fn draw_line(&mut self, line: Line) {
        //! Draws an individual line to the buffer
        let coords: ((usize, usize), (usize, usize)) = line.into();
        draw_line(line.char, self, coords.0, coords.1);
    }
    pub fn draw_lines(&mut self, lines: Vec<Line>) {
        //! Draws lines to the buffer. The first lines in the vector will be drawn first.
        //! # Example
        //! ```
        //! let mut buf = CharBuffer::new(10, 10);
        //! let lines = vec![
        //!     Line {
        //!         char: '+',
        //!         points: (vec2!(2.0, 2.0), vec2!(8.0, 8.0)),
        //!     },
        //!     Line {
        //!         char: '=',
        //!         points: (vec2!(2.0, 8.0), vec2!(8.0, 2.0)),
        //!     },
        //! ];
        //! buf.draw_lines(lines);
        //! println!("{buf}");
        //! ```
        for line in lines {
            self.draw_line(line);
        }
    }
}

fn draw_line(
    char: char,
    buf: &mut CharBuffer,
    mut start_coords: (usize, usize),
    mut end_coords: (usize, usize),
) {
    //! The lower level function for drawing lines. Works, but its best to use higher level as it eliviates the jank of the usize params

    if start_coords.0 > end_coords.0 {
        //Eliminates left-pointing lines
        let b = start_coords;
        start_coords = end_coords;
        end_coords = b;
    }

    let slope = (end_coords.1 as f32 - start_coords.1 as f32)
        / (end_coords.0 as f32 - start_coords.0 as f32);

    if slope < -1.0 {
        //Down vertical
        draw_vertical(char, buf, start_coords, end_coords, false);
    } else if slope > 1.0 {
        //up vertical
        draw_vertical(char, buf, start_coords, end_coords, true);
    } else {
        //right horizontal
        draw_horizontal(char, buf, start_coords, end_coords);
    }
}

fn draw_vertical(
    char: char,
    buf: &mut CharBuffer,
    start_coords: (usize, usize),
    end_coords: (usize, usize),
    is_up: bool,
) {
    let inv_slope = (end_coords.0 as f32 - start_coords.0 as f32)
        / (end_coords.1 as f32 - start_coords.1 as f32);

    if is_up {
        let equation =
            |y: usize| (inv_slope * (y - start_coords.1) as f32 + start_coords.0 as f32) as usize;
        for y in start_coords.1..=end_coords.1 {
            buf.set_char(equation(y), y, char);
        }
    } else {
        let equation =
            |y: usize| (inv_slope * (y - end_coords.1) as f32 + end_coords.0 as f32) as usize;
        for y in end_coords.1..=start_coords.1 {
            buf.set_char(equation(y), y, char);
        }
    }
}

fn draw_horizontal(
    char: char,
    buf: &mut CharBuffer,
    start_coords: (usize, usize),
    end_coords: (usize, usize),
) {
    let slope = (end_coords.1 as f32 - start_coords.1 as f32)
        / (end_coords.0 as f32 - start_coords.0 as f32);

    let equation =
        |x: usize| (slope * (x - start_coords.0) as f32 + start_coords.1 as f32) as usize;

    for x in start_coords.0..=end_coords.0 {
        buf.set_char(x, equation(x), char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn line_conversion() {
        let line = Line {
            char: 'x',
            points: (vec2!(-1.0, 5.0), vec2!(3.0, 4.0)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            ((0, 5), (3, 4))
        );
        let line = Line {
            char: 'x',
            points: (vec2!(3.0, 4.0), vec2!(1.0, -2.0)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            ((2, 0), (3, 4))
        );
        let line = Line {
            char: 'x',
            points: (vec2!(1.0, 5.0), vec2!(3.0, -1.0)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            ((1, 5), (3, 0))
        );
        let line = Line {
            char: 'x',
            points: (vec2!(-1.0, 5.0), vec2!(3.0, 5.0)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            ((0, 5), (3, 5))
        );
        let line = Line {
            char: 'x',
            points: (vec2!(1.0, -2.0), vec2!(1.0, 3.0)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            ((1, 0), (1, 3))
        );
        let line = Line {
            char: 'x',
            points: (vec2!(1.0, -2.0), vec2!(1.0, -3.0)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            (
                (4_000_000_000, 4_000_000_000),
                (4_000_000_000, 4_000_000_000)
            )
        );
        let line = Line {
            char: 'x',
            points: (vec2!(-1.0, 2.0), vec2!(-4.0, 3.0)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            (
                (4_000_000_000, 4_000_000_000),
                (4_000_000_000, 4_000_000_000)
            )
        );
        let line = Line {
            char: 'x',
            points: (vec2!(-1.0, 2.0), vec2!(-4.0, -3.0)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            (
                (4_000_000_000, 4_000_000_000),
                (4_000_000_000, 4_000_000_000)
            )
        );
        let line = Line {
            char: 'x',
            points: (vec2!(1.3, 2.5), vec2!(4.3, 3.9)),
        };
        assert_eq!(
            Into::<((usize, usize), (usize, usize))>::into(line),
            ((1, 3), (4, 4))
        );
    }
}
