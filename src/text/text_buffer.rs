use gtk::{TextBuffer, TextIter, TextTag, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub start: i32,
    pub end: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextWithTags {
    pub text: String,
    pub tags: Vec<Tag>,
}

impl Default for TextWithTags {
    fn default() -> Self {
        Self {
            text: String::new(),
            tags: vec![],
        }
    }
}

impl TextWithTags {
    pub fn from_str(text: &str) -> Self {
        Self {
            text: text.to_string(),
            tags: vec![],
        }
    }
    pub fn as_text_buffer(&self) -> TextBuffer {
        let text_buffer = TextBuffer::new(None);
        text_buffer.create_tag(Some("highlight"), &[("background", &"#FEF3AC")]);
        text_buffer.create_tag(Some("bold"), &[("weight", &800)]);
        text_buffer.set_text(&self.text);

        for tag in self.tags.iter() {
            text_buffer.apply_tag_by_name(
                tag.name.as_str(),
                &text_buffer.iter_at_offset(tag.start),
                &text_buffer.iter_at_offset(tag.end),
            );
        }

        text_buffer
    }

    pub fn clipboard_text(&self) -> String {
        let mut text_parts = vec![];
        let mut last_offset = 0;

        let chars = self.text.chars().collect::<Vec<_>>();

        for offset in 0..chars.len() + 1 {
            let tag_parts: Vec<_> = self
                .tags
                .iter()
                .filter(|tag| tag.start == offset as i32 || tag.end == offset as i32)
                .filter_map(|t| match t.name.as_str() {
                    "bold" => Some("**".to_string()),
                    "highlight" => Some("==".to_string()),
                    _ => None,
                })
                .collect();

            if tag_parts.len() > 0 {
                text_parts.push(chars[last_offset..offset].iter().collect::<String>());
                text_parts.extend(tag_parts);
                last_offset = offset;
            }
        }
        text_parts.push(chars[last_offset..].iter().collect());
        text_parts.join("")
    }

    pub fn find_tag(tag: &TextTag, mut iter: TextIter, end: TextIter) -> Option<Tag> {
        if !iter.starts_tag(Some(tag)) {
            return None;
        }
        let start = iter.offset();
        while iter.offset() <= end.offset() {
            if !iter.tags().iter().any(|t| t.name() == tag.name()) {
                return Some(Tag {
                    start,
                    end: iter.offset(),
                    name: tag.name().unwrap().to_string(),
                });
            }
            iter.forward_char();
        }

        Some(Tag {
            start,
            end: end.offset(),
            name: tag.name().unwrap().to_string(),
        })
    }

    pub fn from(text_buffer: &TextBuffer, mut start: TextIter, end: TextIter) -> Self {
        let text = text_buffer.text(&start, &end, false).to_string();
        let start_offset = start.offset();
        let mut tags = Vec::new();

        while start.offset() < end.offset() {
            for tag in start
                .tags()
                .iter()
                .filter(|&tag| start.starts_tag(Some(tag)))
            {
                if let Some(tag) = Self::find_tag(tag, start.clone(), end.clone()) {
                    tags.push(Tag {
                        start: tag.start - start_offset,
                        end: tag.end - start_offset,
                        name: tag.name,
                    });
                }
            }
            start.forward_char();
        }

        Self { text, tags }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_text() {
        // Test case 1: Bold text at the start
        let text_with_tags = TextWithTags {
            text: String::from("Bold text here"),
            tags: vec![Tag {
                start: 0,
                end: 4,
                name: String::from("bold"),
            }],
        };
        assert_eq!(text_with_tags.clipboard_text(), "**Bold** text here");

        // Test case 2: Highlight text in the middle
        let text_with_tags = TextWithTags {
            text: String::from("Some highlighted text."),
            tags: vec![Tag {
                start: 5,
                end: 16,
                name: String::from("highlight"),
            }],
        };
        assert_eq!(
            text_with_tags.clipboard_text(),
            "Some ==highlighted== text."
        );

        // Test case 3: Multiple tags
        let text_with_tags = TextWithTags {
            text: String::from("Bold and highlighted"),
            tags: vec![
                Tag {
                    start: 0,
                    end: 4,
                    name: String::from("bold"),
                },
                Tag {
                    start: 9,
                    end: 20,
                    name: String::from("highlight"),
                },
            ],
        };
        assert_eq!(
            text_with_tags.clipboard_text(),
            "**Bold** and ==highlighted=="
        );

        // Test case 4: Overlapping tags (should be ignored)
        let text_with_tags = TextWithTags {
            text: String::from("Overlapping tags"),
            tags: vec![
                Tag {
                    start: 0,
                    end: 10,
                    name: String::from("bold"),
                },
                Tag {
                    start: 5,
                    end: 15,
                    name: String::from("highlight"),
                },
            ],
        };
        assert_eq!(text_with_tags.clipboard_text(), "**Overl==appin**g tag==s");

        let text_with_tags = TextWithTags {
            text: String::from("한글 텍스트"),
            tags: vec![Tag {
                start: 1,
                end: 2,
                name: String::from("bold"),
            }],
        };
        assert_eq!(text_with_tags.clipboard_text(), "한**글** 텍스트");
    }
}
