/// Represents a single element within a function/struct namespace path.
#[derive(Debug, Clone, PartialEq)]
pub struct IRPathElement {
    /// The identifier of this namespace path element.
    pub identifier: String,

    /// Whether or not this element is exported to cousin paths.
    pub exported: bool,
}
