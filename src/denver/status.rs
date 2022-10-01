use std::cmp::max;
use std::fmt::Display;

const PADDING: usize = 2;

pub struct Containers<'a> {
    pub data: Vec<Container<'a>>,
}

impl<'a> Containers<'a> {
    pub fn new() -> Self {
        Containers {
            data: vec![Container::new(
                "CONTAINER ID",
                "NAME",
                "IMAGE",
                "STATE",
                "STATUS",
            )],
        }
    }

    fn find_lengths(&self) -> (usize, usize, usize, usize) {
        let mut name = 0;
        let mut image = 0;
        let mut state = 0;
        let mut status = 0;

        for data in &self.data {
            name = max(data.name.len(), name);
            image = max(data.image.len(), image);
            state = max(data.state.len(), state);
            status = max(data.status.len(), status);
        }
        (
            name + PADDING,
            image + PADDING,
            state + PADDING,
            status + PADDING,
        )
    }

    pub fn push(&mut self, c: Container<'a>) {
        self.data.push(c);
    }
}

impl<'a> Display for Containers<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, image, state, status) = self.find_lengths();
        let id = 12 + PADDING;

        for line in &self.data {
            writeln!(
                f,
                "{:id$}{:name$}{:image$}{:state$}{:status$}",
                line.id,
                line.name,
                line.image,
                line.state.to_uppercase(),
                line.status
            )?;
        }
        Ok(())
    }
}

pub struct Container<'a> {
    id: &'a str,
    name: &'a str,
    image: &'a str,
    state: &'a str,
    status: &'a str,
}

impl<'a> Container<'a> {
    pub fn new(
        id: &'a str,
        name: &'a str,
        image: &'a str,
        state: &'a str,
        status: &'a str,
    ) -> Self {
        Container {
            id,
            name,
            image,
            state,
            status,
        }
    }
}
