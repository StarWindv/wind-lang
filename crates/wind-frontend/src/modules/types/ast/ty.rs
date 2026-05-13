#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(String),
    Fn {
        params: Vec<Type>,
        ret: Box<Type>,
    },
}
