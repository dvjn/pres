use pulldown_cmark::{Event, Options, Parser};

#[derive(Clone)]
pub struct Slide {
    pub raw: String,
}

#[derive(Clone)]
pub struct Section {
    pub slides: Vec<Slide>,
}

enum SplitKind {
    Horizontal, // --- (Rule with 3 dashes)
    Vertical,   // ----- (Rule with 5 dashes)
}

struct SplitPoint {
    kind: SplitKind,
    start: usize,
    end: usize,
}

fn collect_split_points(input: &str) -> Vec<SplitPoint> {
    let parser = Parser::new_ext(input, Options::all());
    let mut points = Vec::new();

    for (event, range) in parser.into_offset_iter() {
        if let Event::Rule = event {
            let raw = input[range.start..range.end].trim();
            let kind = if raw.chars().all(|c| c == '-') && raw.len() >= 5 {
                SplitKind::Vertical
            } else {
                SplitKind::Horizontal
            };
            points.push(SplitPoint {
                kind,
                start: range.start,
                end: range.end,
            });
        }
    }

    points
}

pub fn parse(input: &str) -> Vec<Section> {
    let points = collect_split_points(input);

    if points.is_empty() {
        let trimmed = input.trim();
        return if trimmed.is_empty() {
            vec![]
        } else {
            vec![Section {
                slides: vec![Slide {
                    raw: trimmed.to_string(),
                }],
            }]
        };
    }

    let mut sections: Vec<Section> = vec![Section { slides: vec![] }];
    let mut prev = 0usize;

    for point in &points {
        let trimmed = input[prev..point.start].trim();
        if !trimmed.is_empty() {
            sections.last_mut().unwrap().slides.push(Slide {
                raw: trimmed.to_string(),
            });
        }
        prev = point.end;
        if matches!(point.kind, SplitKind::Horizontal) {
            sections.push(Section { slides: vec![] });
        }
    }

    let trimmed = input[prev..].trim();
    if !trimmed.is_empty() {
        sections.last_mut().unwrap().slides.push(Slide {
            raw: trimmed.to_string(),
        });
    }

    // Drop sections that ended up with no slides
    sections.retain(|s| !s.slides.is_empty());
    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_rule_splits_into_separate_sections() {
        let sections = parse("A\n\n---\n\nB");
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].slides.len(), 1);
        assert_eq!(sections[1].slides.len(), 1);
    }

    #[test]
    fn vertical_marker_splits_into_slides_within_same_section() {
        let sections = parse("A\n\n-----\n\nB");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slides.len(), 2);
    }

    #[test]
    fn horizontal_rule_resets_vertical_slide_index() {
        // After ---, the new section starts at slide 0 again
        let sections = parse("A\n\n-----\n\nB\n\n---\n\nC");
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].slides.len(), 2);
        assert_eq!(sections[1].slides.len(), 1);
    }

    #[test]
    fn dashes_inline_in_text_are_not_a_horizontal_split() {
        // A line with extra dashes or dashes mid-paragraph is not a Rule
        let sections = parse("some --- text");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slides.len(), 1);
    }

    #[test]
    fn multiple_vertical_splits_produce_multiple_slides() {
        let sections = parse("A\n\n-----\n\nB\n\n-----\n\nC");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slides.len(), 3);
    }

    #[test]
    fn vertical_split_at_start_produces_one_slide() {
        // Leading ----- with no prior content produces no empty slide
        let sections = parse("-----\n\nA");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slides.len(), 1);
    }

    #[test]
    fn vertical_split_at_end_produces_one_slide() {
        let sections = parse("A\n\n-----");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slides.len(), 1);
    }

    #[test]
    fn horizontal_split_at_start_produces_one_section() {
        let sections = parse("---\n\nA");
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn horizontal_split_at_end_produces_one_section() {
        let sections = parse("A\n\n---");
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn slide_raw_content_preserves_full_markdown() {
        let sections = parse("# Title\n\n- item\n\n> quote");
        assert_eq!(sections[0].slides[0].raw, "# Title\n\n- item\n\n> quote");
    }
}
