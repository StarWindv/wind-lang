use super::ty::WindType;

#[derive(Debug, Clone, PartialEq)]
pub struct WindFnParam {
    pub name: String,
    pub ty: Option<WindType>,
}
