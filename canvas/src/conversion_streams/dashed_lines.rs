use crate::draw::*;

use flo_curves::bezier::*;
use flo_curves::bezier::path::*;
use flo_stream::*;
use futures::prelude::*;

use std::iter;

///
/// Converts a bezier path to a set of paths by a dash patter
///
pub fn path_to_dashed_lines<PathIn, PathOut, DashPattern>(path_in: PathIn, dash_pattern: DashPattern) -> Vec<PathOut> 
where
PathIn:         BezierPath,
PathOut:        BezierPathFactory<Point=PathIn::Point>,
DashPattern:    Clone+Iterator<Item=f64> {
    // Create the resulting set of paths (most will have just a single curve in them)
    let mut output_paths        = vec![];

    // Cycle the dash pattern
    let mut dash_pattern        = dash_pattern.cycle();

    // Fetch the first length
    let mut remaining_length    = dash_pattern.next().unwrap();

    // We alternate between drawing and not drawing dashes
    let mut draw_dash           = true;

    // Generate dashed lines for each path segment
    let mut start_point         = path_in.start_point();
    let mut current_path_start  = start_point;
    let mut current_path_points = vec![];

    for (cp1, cp2, end_point) in path_in.points() {
        // Create a curve for this section
        let curve                   = Curve::from_points(start_point, (cp1, cp2), end_point);

        if remaining_length <= 0.0 {
            remaining_length        = dash_pattern.next().unwrap();
            draw_dash               = !draw_dash;
        }

        // Walk it, starting with the remaining length and then moving on according to the dash pattern
        let dash_pattern            = &mut dash_pattern;
        let mut dash_pattern_copy   = iter::once(remaining_length).chain(dash_pattern.clone());
        let dash_pattern            = iter::once(remaining_length).chain(dash_pattern);

        for section in walk_curve_evenly(&curve, 1.0, 0.05).vary_by(dash_pattern) {
            // The copied dash pattern will get the expected length for this dash
            let next_length                 = dash_pattern_copy.next().unwrap();

            // walk_curve_evenly uses chord lengths (TODO: arc lengths would be better)
            let section_length              = chord_length(&section);

            // Update the remaining length
            remaining_length                = next_length - section_length;

            // Add the dash to the current path
            let (section_cp1, section_cp2)  = section.control_points();
            let section_end_point           = section.end_point();
            current_path_points.push((section_cp1, section_cp2, section_end_point));

            // If there's enough space for the whole dash, invert the 'draw_dash' state and add the current path to the result
            if remaining_length < 0.1 {
                // Add this dash to the output
                if draw_dash {
                    output_paths.push(PathOut::from_points(current_path_start, current_path_points));
                }

                // Clear the current path
                current_path_start  = section_end_point;
                current_path_points = vec![];

                // Reset for the next dash
                remaining_length    = 0.0;
                draw_dash           = !draw_dash;
            }
        }

        // The start point of the next curve in this path is the end point of this one
        start_point = end_point;
    }

    output_paths
}

///
/// Converts dashed line stroke operations into separate lines
///
pub fn drawing_without_dashed_lines<InStream: 'static+Send+Unpin+Stream<Item=Draw>>(draw_stream: InStream) -> impl Send+Unpin+Stream<Item=Draw> {
    generator_stream(move |yield_value| async move {
        let mut draw_stream = draw_stream;

        while let Some(drawing) = draw_stream.next().await {
            // Pass the drawing on
            yield_value(drawing).await;
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    use futures::stream;
    use futures::executor;

    #[test]
    fn pass_through_normal_path() {
        let input_drawing = vec![
            Draw::NewPath,
            Draw::Move(10.0, 10.0),
            Draw::Line(10.0, 100.0),
            Draw::Line(100.0, 100.0),
            Draw::Line(100.0, 10.0),
            Draw::ClosePath
        ];

        executor::block_on(async move {
            let without_dashed_lines    = drawing_without_dashed_lines(stream::iter(input_drawing.into_iter()));
            let output_drawing          = without_dashed_lines.collect::<Vec<_>>().await;

            assert!(output_drawing == vec![
                Draw::NewPath,
                Draw::Move(10.0, 10.0),
                Draw::Line(10.0, 100.0),
                Draw::Line(100.0, 100.0),
                Draw::Line(100.0, 10.0),
                Draw::ClosePath
            ]);
        });
    }
}