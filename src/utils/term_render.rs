/*
 * The plan is to make something that can be rendered
 * The eval code forgets about all this and just produces cells to be rendered
 * I can see there are:
 *  - Chunks
 *  - Titles
 *  - Cells
 *
 * The titles are optional, it seems
 * It'd be quite fun to give it a comical name
 * Something like "caterpillar"
 * How about "column" "block" "heading" "cell" "figure" "comment"
 * Let's abstract a touch. Instead of "heading", we have "segment" that represents the hoz line.
 * That means we can reuse the concept for the bottom blocks that have no headings,
 * but do have the hoz line
 *
 * Huh turns out the model is really easy to represent haha
 */

use itertools::Itertools;

use std::{fmt::Display, iter::once};


pub struct Cell {
    pub figure: String,
    pub comment: String,
} 

pub struct Segment(pub Vec<Cell>);
pub struct Block(pub Vec<Segment>); 
pub struct Column(pub Vec<Block>);


impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Clone)]
        enum RenderType<'a> {
            Before,
            Segment(&'a Segment),
            Divider,
            After,
        }

        let segments = self.0.iter().map(RenderType::Segment);
        let with_dividers = Itertools::intersperse(segments, RenderType::Divider);
        let with_bookends = once(RenderType::Before).chain(with_dividers).chain(once(RenderType::After));

        for render in with_bookends {
            match render {
                RenderType::Before => { f.write_str("    ┌───────┐\n")?; },
                RenderType::Segment(segment) => {
                    for Cell { figure, comment } in &segment.0 {
                        f.write_fmt(format_args!("    │ {:<5} │ {}\n", figure, comment))?;
                    }
                },
                RenderType::Divider => { f.write_str("    ├───────┤\n")?; },
                RenderType::After => { f.write_str("    └───────┘\n")?; },
            }
        }

        Ok(())
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for block in &self.0 {
            f.write_fmt(format_args!("{block}"))?;
        }

        Ok(())
    }
}

#[test]
fn check_display_feature() {
    // Must be run using
    // cargo t -- --nocapture
    // To see output
    let column = Column(vec![
        Block(vec![
            Segment(vec![
                Cell {
                    figure: "+15m".into(),
                    comment: "A comment".into(),
                },
                Cell {
                    figure: "-40m".into(),
                    comment: "Another comment".into(),
                }
            ]),
            Segment(vec![
                Cell {
                    figure: "+1:30".into(),
                    comment: "Lots of time here".into()
                }
            ])
        ]),
        Block(vec![
            Segment(vec![
                Cell {
                    figure: "+3:00".into(),
                    comment: "By itself".into(),
                }
            ]),
            Segment(vec![
                Cell {
                    figure: "-6:00".into(),
                    comment: "Alone and yet together".into(),
                }
            ]),
            Segment(vec![
                Cell {
                    figure: "+9:00".into(),
                    comment: "Like two passing ships".into(),
                }
            ]),
        ])
    ]);

    println!("{column}");
}

