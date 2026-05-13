#[derive(Debug, Clone, PartialEq)]
pub enum WindType {
    Named(String),
    Fn {
        params: Vec<WindType>,
        ret: Box<WindType>,
    },
}
