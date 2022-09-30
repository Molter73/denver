use std::cmp::max;
use std::fmt::Display;

const PADDING: usize = 2;

pub struct RunningStatus<'a> {
    pub data: Vec<StatusInfo<'a>>,
}

impl<'a> RunningStatus<'a> {
    pub fn new() -> Self {
        RunningStatus { data: Vec::new() }
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
}

impl<'a> Display for RunningStatus<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, image, state, status) = self.find_lengths();
        let id = 12 + PADDING;
        let mut output = String::new();

        for line in &self.data {
            output += &format!(
                "{:id$}{:name$}{:image$}{:state$}{:status$}\n",
                line.id,
                line.name,
                line.image,
                line.state.to_uppercase(),
                line.status
            );
        }
        write!(f, "{}", output)
    }
}

pub struct StatusInfo<'a> {
    id: &'a str,
    name: &'a str,
    image: &'a str,
    state: &'a str,
    status: &'a str,
}

impl<'a> StatusInfo<'a> {
    pub fn new(
        id: &'a str,
        name: &'a str,
        image: &'a str,
        state: &'a str,
        status: &'a str,
    ) -> Self {
        StatusInfo {
            id,
            name,
            image,
            state,
            status,
        }
    }
}
