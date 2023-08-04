use crate::{get_file_path, error::GlobalError};

/// A tag is like a category. pnchs are grouped by tags.
#[derive(Clone, Debug)]
pub struct Tag {
    // The id of the tag.
    //
    // The id is it's index in the tags file. The first tag has the id 0, etc...
    pub id: u32,
    /// The value of the tag.
    ///
    /// The tag can be a maximum of 24 chars long and is always saved as an 24 chars long value.
    pub tag: String,
}

impl Tag {
    /// size of the id field in bytes
    pub const ID_SIZE: usize = 4;
    /// size of the description field in bytes
    const TAG_SIZE: usize = 24;
    /// total size of each tag in bytes
    const SIZE: usize = Self::ID_SIZE + Self::TAG_SIZE;

    pub fn none() -> Self {
        Self {
            id: u32::MAX,
            tag: String::new()
        }
    }
}

impl std::convert::TryFrom<&[u8]> for Tag {
    type Error = GlobalError;
    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() != Self::SIZE {
            return Err(GlobalError::wrong_byte_len("tag", buffer.len(), Self::SIZE));
        }
        let (id_bytes, tag_bytes) = buffer.split_at(Self::ID_SIZE);
        let tag_bytes = tag_bytes
            .iter()
            .copied()
            .filter(|&c| c != 0)
            .collect::<Vec<u8>>();
        let id_bytes = id_bytes
            .try_into()
            .expect("split_at already panics when wrong size");
        Ok(Self {
            id: u32::from_le_bytes(id_bytes),
            tag: String::from_utf8(tag_bytes)?
        })
    }
}

impl From<&Tag> for Vec<u8> {
    fn from(tag: &Tag) -> Self {
        let mut buffer = Vec::with_capacity(Tag::SIZE);
        buffer.extend_from_slice(&tag.id.to_le_bytes());
        buffer.extend_from_slice(tag.tag.as_bytes());
        buffer.append(&mut vec![0; Tag::SIZE - buffer.len()]);
        buffer
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.tag)
    }
}

/// A group of tags
pub struct Tags(Vec<Tag>);

impl Tags {
    const TAGS_FILE_NAME: &'static str = "tags.db";

    pub fn load() -> Result<Self, GlobalError> {
        let path = get_file_path(Self::TAGS_FILE_NAME)?;
        Ok(Self(std::fs::read(path)
            .map_err(|_| GlobalError::fs("load", "tags"))?
            .chunks_exact(Tag::SIZE)
            .into_iter()
            .map(|chunk| Tag::try_from(chunk))
            .collect::<Result<Vec<Tag>, GlobalError>>()?))
    }

    pub fn get_or_insert(&mut self, tag_name: String) -> Tag {
        match self.0.iter().find(|tag| tag.tag == tag_name) {
            Some(tag) => tag.clone(),
            _ => {
                let tag = Tag {
                    id: self.0.len() as u32,
                    tag: tag_name
                };
                self.0.push(tag.clone());
                tag
            }
        }
    }

    pub fn get(&self, id: u32) -> Option<Tag> {
        self.0.get(id as usize).cloned()
    }

    pub fn save(&self) -> Result<(), GlobalError> {
        let path = get_file_path(Self::TAGS_FILE_NAME)?;
        let content = self.0
            .iter()
            .map(|tag| Vec::from(tag))
            .flatten()
            .collect::<Vec<u8>>();
        std::fs::write(path, content)
            .map_err(|_| GlobalError::fs("save", "tags"))?;
        Ok(())
    }
}
