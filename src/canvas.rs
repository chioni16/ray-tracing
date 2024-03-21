use crate::colour::*;
use std::path::Path;

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec<Colour>>,
}

impl Canvas {
    pub fn new(width: usize, height: usize, colour: Colour) -> Self {
        Self {
            width,
            height,
            pixels: vec![vec![colour; width]; height],
        }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, colour: Colour) {
        self.pixels[y][x] = colour;
    }

    fn to_ppm(&self) -> String {
        let mut s = format!("P3\n{} {}\n{}\n", self.width, self.height, 255);

        for row in self.pixels.iter() {
            for colour in row {
                let colour = colour.0 .0;
                let colour = colour
                    .iter()
                    .map(|c| (c.max(0.0).min(1.0) * 255.0).round())
                    // .map(|c| (c * 255.0).round())
                    .collect::<Vec<_>>();
                s.push_str(format!("{} {} {} ", colour[0], colour[1], colour[2]).as_str());
            }
            s.pop();
            s.push('\n');
        }

        s
    }

    pub fn to_file(&self, path: &Path) -> std::io::Result<()> {
        let a = self.to_ppm();
        std::fs::write(path, a)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_ppm_test() {
        let canvas = Canvas {
            width: 1,
            height: 1,
            pixels: vec![vec![Colour::new(
                0.4947756324442701,
                0.17761176549281493,
                0.6089546245467938,
            )]],
        };
        assert_eq!(canvas.to_ppm(), "P3\n1 1\n255\n126 45 155\n");

        let canvas = Canvas {
            width: 1,
            height: 1,
            pixels: vec![vec![Colour::new(0.078, 0.028, 0.096)]],
        };
        assert_eq!(canvas.to_ppm(), "P3\n1 1\n255\n20 7 24\n");
    }
}
