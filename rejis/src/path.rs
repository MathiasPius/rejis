use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum PathElement {
    Field(&'static str),
    Index(usize),
}

impl Display for PathElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathElement::Field(field) => {
                write!(f, ".{field}")
            }
            PathElement::Index(index) => {
                write!(f, "[{index}]")
            }
        }
    }
}

impl From<usize> for PathElement {
    fn from(value: usize) -> Self {
        PathElement::Index(value)
    }
}

impl From<&'static str> for PathElement {
    fn from(value: &'static str) -> Self {
        PathElement::Field(value)
    }
}

#[derive(Default, Debug, Clone)]
pub struct Path(Vec<PathElement>);

impl Path {
    pub fn join<P: Into<PathElement>>(&self, element: P) -> Self {
        let mut path = self.0.clone();
        path.push(element.into());
        Path(path)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("$")?;
        for field in &self.0 {
            write!(f, "{field}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Path;

    #[test]
    fn build_path() {
        let path = Path::default().join("pets").join(0).join("name");

        assert_eq!(path.to_string(), "$.pets[0].name");
    }
}
