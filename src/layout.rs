use eframe::epaint::Pos2;

pub const HSTEP: f32 = 13.;
pub const VSTEP: f32 = 15.;

pub struct DisplayListItem {
    pub pos: Pos2,
    pub ch: char,
}

pub fn calculate_draw_list(input: &str, width: f32) -> Vec<DisplayListItem> {
    let mut pos = Pos2::ZERO;

    input
        .chars()
        .map(|ch| {
            let layout = DisplayListItem { pos, ch };
            pos.x += HSTEP;
            if pos.x >= width {
                pos.x = 0.;
                pos.y += VSTEP;
            }

            layout
        })
        .collect()
}
