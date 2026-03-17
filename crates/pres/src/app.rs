use crate::parser::Section;

pub struct App {
    pub slide_counts: Vec<usize>,
    pub col: usize,
    pub row: usize,
    pub filename: String,
}

impl App {
    pub fn new(sections: &[Section], filename: String) -> Self {
        Self {
            slide_counts: sections.iter().map(|s| s.slides.len()).collect(),
            col: 0,
            row: 0,
            filename,
        }
    }

    pub fn new_at(sections: &[Section], filename: String, col: usize, row: usize) -> Self {
        let slide_counts: Vec<usize> = sections.iter().map(|s| s.slides.len()).collect();
        let col = col.min(slide_counts.len().saturating_sub(1));
        let row = row.min(
            slide_counts
                .get(col)
                .copied()
                .unwrap_or(1)
                .saturating_sub(1),
        );
        Self {
            slide_counts,
            col,
            row,
            filename,
        }
    }

    pub fn next(&mut self) {
        if self.row + 1 < self.slide_counts[self.col] {
            self.row += 1;
        } else if self.col + 1 < self.slide_counts.len() {
            self.col += 1;
            self.row = 0;
        }
    }

    pub fn prev(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        } else if self.col > 0 {
            self.col -= 1;
            self.row = self.slide_counts[self.col].saturating_sub(1);
        }
    }

    pub fn right(&mut self) {
        if self.col + 1 < self.slide_counts.len() {
            self.col += 1;
            self.row = 0;
        }
    }

    pub fn left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
            self.row = 0;
        }
    }

    pub fn down(&mut self) {
        if self.row + 1 < self.slide_counts[self.col] {
            self.row += 1;
        }
    }

    pub fn up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        }
    }

    pub fn first(&mut self) {
        self.col = 0;
        self.row = 0;
    }

    pub fn last(&mut self) {
        self.col = self.slide_counts.len().saturating_sub(1);
        self.row = self.slide_counts[self.col].saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_app(layout: &[usize]) -> App {
        App {
            slide_counts: layout.to_vec(),
            col: 0,
            row: 0,
            filename: String::new(),
        }
    }

    #[test]
    fn next_advances_within_section() {
        let mut app = make_app(&[3]);
        app.next();
        assert_eq!((app.col, app.row), (0, 1));
        app.next();
        assert_eq!((app.col, app.row), (0, 2));
    }

    #[test]
    fn next_wraps_to_next_section() {
        let mut app = make_app(&[2, 2]);
        app.next();
        app.next(); // end of section 0, should go to section 1
        assert_eq!((app.col, app.row), (1, 0));
    }

    #[test]
    fn next_clamps_at_last_slide() {
        let mut app = make_app(&[1, 1]);
        app.next(); // move to section 1
        app.next(); // already at last slide
        assert_eq!((app.col, app.row), (1, 0));
    }

    #[test]
    fn prev_goes_back_within_section() {
        let mut app = make_app(&[3]);
        app.next();
        app.next();
        app.prev();
        assert_eq!((app.col, app.row), (0, 1));
    }

    #[test]
    fn prev_jumps_to_last_slide_of_previous_section() {
        let mut app = make_app(&[3, 1]);
        app.next(); // (0,1)
        app.next(); // (0,2)
        app.next(); // (1,0)
        app.prev(); // should land on last slide of section 0
        assert_eq!((app.col, app.row), (0, 2));
    }

    #[test]
    fn prev_clamps_at_first_slide() {
        let mut app = make_app(&[2]);
        app.prev();
        assert_eq!((app.col, app.row), (0, 0));
    }

    #[test]
    fn right_moves_to_next_section_row_zero() {
        let mut app = make_app(&[2, 2]);
        app.down();
        app.right();
        assert_eq!((app.col, app.row), (1, 0));
    }

    #[test]
    fn right_clamps_at_last_section() {
        let mut app = make_app(&[1, 1]);
        app.right();
        app.right(); // already at last section
        assert_eq!((app.col, app.row), (1, 0));
    }

    #[test]
    fn left_moves_to_previous_section_row_zero() {
        let mut app = make_app(&[2, 2]);
        app.right();
        app.down();
        app.left();
        assert_eq!((app.col, app.row), (0, 0));
    }

    #[test]
    fn left_clamps_at_first_section() {
        let mut app = make_app(&[2]);
        app.left();
        assert_eq!((app.col, app.row), (0, 0));
    }

    #[test]
    fn down_moves_within_section() {
        let mut app = make_app(&[3]);
        app.down();
        assert_eq!((app.col, app.row), (0, 1));
    }

    #[test]
    fn down_does_not_cross_section_boundary() {
        let mut app = make_app(&[1, 1]);
        app.down();
        assert_eq!((app.col, app.row), (0, 0));
    }

    #[test]
    fn up_moves_within_section() {
        let mut app = make_app(&[3]);
        app.down();
        app.down();
        app.up();
        assert_eq!((app.col, app.row), (0, 1));
    }

    #[test]
    fn up_clamps_at_first_row() {
        let mut app = make_app(&[2]);
        app.up();
        assert_eq!((app.col, app.row), (0, 0));
    }

    #[test]
    fn first_goes_to_start() {
        let mut app = make_app(&[2, 3]);
        app.right();
        app.down();
        app.down();
        app.first();
        assert_eq!((app.col, app.row), (0, 0));
    }

    #[test]
    fn last_goes_to_last_slide_of_last_section() {
        let mut app = make_app(&[2, 3]);
        app.last();
        assert_eq!((app.col, app.row), (1, 2));
    }
}
