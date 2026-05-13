#[derive(Debug, Clone, PartialEq)]
pub enum WindType {
    Named(String),
    Generic {
        base: String,
        args: Vec<WindType>,
    },
    Fn {
        params: Vec<WindType>,
        ret: Box<WindType>,
    },
    SelfType,
}
