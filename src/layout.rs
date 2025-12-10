use crate::ocr_engine::OcrBlock;

/// Sort OCR blocks in reading order (top-to-bottom, left-to-right).
pub fn sort_by_reading_order(blocks: &mut Vec<OcrBlock>) {
    blocks.sort_by(|a, b| {
        let (ay, ax) = bbox_top_left(a);
        let (by, bx) = bbox_top_left(b);

        let y_delta = ay.abs_diff(by);
        if y_delta <= 12 {
            ax.cmp(&bx)
        } else {
            ay.cmp(&by)
        }
    });
}

fn bbox_top_left(block: &OcrBlock) -> (u32, u32) {
    block
        .bbox
        .as_ref()
        .map(|b| (b.y, b.x))
        .unwrap_or((0, 0))
}
