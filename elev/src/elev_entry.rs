use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum ElevEntryError {
    MissingField(&'static str),
    ParseIntError(&'static str, ParseIntError),
    InvalidCount(&'static str, usize, usize),
}

impl fmt::Display for ElevEntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElevEntryError::MissingField(field) => write!(f, "Missing field: {}", field),
            ElevEntryError::ParseIntError(field, error) => {
                write!(f, "Failed to parse {} as integer: {}", field, error)
            }
            ElevEntryError::InvalidCount(field, expected, actual) => write!(
                f,
                "Invalid {} count: expected {}, got {}",
                field, expected, actual
            ),
        }
    }
}

impl std::error::Error for ElevEntryError {}

#[derive(Debug)]
pub struct ElevEntry {
    pub page_x: i32,
    pub page_z: i32,
    pub node_x: u8, /* 0 - 127 */
    pub node_z: u8, /* 0 - 127 */
    pub node_radius: u8,
    pub texture_ids: Vec<u32>,
    pub heights: Vec<i32>,
}

impl ElevEntry {
    pub fn from_line(line: impl AsRef<str>) -> Result<Self, ElevEntryError> {
        let line = line.as_ref();
        let mut parts = line.split_whitespace();

        let mut parse_field = |field: &'static str| -> Result<&str, ElevEntryError> {
            parts.next().ok_or(ElevEntryError::MissingField(field))
        };

        let page_x = parse_field("page_x")?
            .parse::<i32>()
            .map_err(|e| ElevEntryError::ParseIntError("page_x", e))?;
        let page_z = parse_field("page_z")?
            .parse::<i32>()
            .map_err(|e| ElevEntryError::ParseIntError("page_z", e))?;
        let node_x = parse_field("node_x")?
            .parse::<u8>()
            .map_err(|e| ElevEntryError::ParseIntError("node_x", e))?;
        let node_z = parse_field("node_z")?
            .parse::<u8>()
            .map_err(|e| ElevEntryError::ParseIntError("node_z", e))?;
        let node_radius = parse_field("node_radius")?
            .parse::<u8>()
            .map_err(|e| ElevEntryError::ParseIntError("node_radius", e))?;

        let texture_count = parse_field("texture_count")?
            .parse::<usize>()
            .map_err(|e| ElevEntryError::ParseIntError("texture_count", e))?;
        let height_count = parse_field("height_count")?
            .parse::<usize>()
            .map_err(|e| ElevEntryError::ParseIntError("height_count", e))?;

        let texture_ids: Result<Vec<u32>, ElevEntryError> = (0..texture_count)
            .map(|_| {
                parse_field("texture_id").and_then(|s| {
                    s.parse::<u32>()
                        .map_err(|e| ElevEntryError::ParseIntError("texture_id", e))
                })
            })
            .collect();
        let texture_ids = texture_ids?;

        let heights: Result<Vec<i32>, ElevEntryError> = (0..height_count)
            .map(|_| {
                parse_field("height").and_then(|s| {
                    s.parse::<i32>()
                        .map_err(|e| ElevEntryError::ParseIntError("height", e))
                })
            })
            .collect();
        let heights = heights?;

        if texture_ids.len() != texture_count {
            return Err(ElevEntryError::InvalidCount(
                "texture",
                texture_count,
                texture_ids.len(),
            ));
        }

        if heights.len() != height_count {
            return Err(ElevEntryError::InvalidCount(
                "height",
                height_count,
                heights.len(),
            ));
        }

        Ok(Self {
            page_x,
            page_z,
            node_x,
            node_z,
            node_radius,
            texture_ids,
            heights,
        })
    }
}
