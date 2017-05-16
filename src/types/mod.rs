/// Type to encapsulate 'one or many' values
#[derive(Debug, PartialEq)] 
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}