pub mod ast;
pub mod tokens;

pub fn byte_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let line = source[..offset].matches('\n').count() + 1;
    let last_nl = source[..offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let col = offset - last_nl + 1;
    (line, col)
}
