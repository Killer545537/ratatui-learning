use ratatui::layout::Rect;

/// Simple util to create a new rectangle which in centered inside another rectangle
/// with a percentage original width and given height
pub fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let width = (r.width * percent_x) / 100;
    let x = r.x + (r.width - width) / 2;
    let y = r.y + (r.height - height) / 2;

    Rect {
        x,
        y,
        width,
        height,
    }
}
