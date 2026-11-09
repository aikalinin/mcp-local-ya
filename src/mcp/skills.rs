use std::fs;
use std::path::Path;

use crate::error::{Result, YamcliError};

struct Skill {
    name: &'static str,
    content: &'static str,
}

const SKILLS: &[Skill] = &[
    Skill {
        name: "yamcli-shared",
        content: include_str!("../../skills/yamcli-shared/SKILL.md"),
    },
    Skill {
        name: "yamcli-mail",
        content: include_str!("../../skills/yamcli-mail/SKILL.md"),
    },
    Skill {
        name: "yamcli-calendar",
        content: include_str!("../../skills/yamcli-calendar/SKILL.md"),
    },
    Skill {
        name: "yamcli-disk",
        content: include_str!("../../skills/yamcli-disk/SKILL.md"),
    },
    Skill {
        name: "yamcli-daily-briefing",
        content: include_str!("../../skills/yamcli-daily-briefing/SKILL.md"),
    },
    Skill {
        name: "yamcli-find-and-read",
        content: include_str!("../../skills/yamcli-find-and-read/SKILL.md"),
    },
    Skill {
        name: "yamcli-reply-with-context",
        content: include_str!("../../skills/yamcli-reply-with-context/SKILL.md"),
    },
    Skill {
        name: "yamcli-attachment-to-disk",
        content: include_str!("../../skills/yamcli-attachment-to-disk/SKILL.md"),
    },
    Skill {
        name: "yamcli-send-file-by-mail",
        content: include_str!("../../skills/yamcli-send-file-by-mail/SKILL.md"),
    },
    Skill {
        name: "yamcli-send-link-by-mail",
        content: include_str!("../../skills/yamcli-send-link-by-mail/SKILL.md"),
    },
    Skill {
        name: "yamcli-publish-file-link",
        content: include_str!("../../skills/yamcli-publish-file-link/SKILL.md"),
    },
    Skill {
        name: "yamcli-revoke-public-link",
        content: include_str!("../../skills/yamcli-revoke-public-link/SKILL.md"),
    },
    Skill {
        name: "yamcli-invite-to-calendar",
        content: include_str!("../../skills/yamcli-invite-to-calendar/SKILL.md"),
    },
];

pub const SKILL_COUNT: usize = 13;

pub fn skill_names() -> Vec<&'static str> {
    SKILLS.iter().map(|s| s.name).collect()
}

pub fn skill_content(name: &str) -> Option<&'static str> {
    SKILLS
        .iter()
        .find(|skill| skill.name == name)
        .map(|skill| skill.content)
}

pub fn skill_description(name: &str) -> Option<String> {
    let content = skill_content(name)?;
    parse_frontmatter_description(content)
}

pub fn skill_prompt_name(name: &str) -> Option<&'static str> {
    match name {
        "yamcli-shared" => Some("shared"),
        "yamcli-mail" => Some("mail"),
        "yamcli-calendar" => Some("calendar"),
        "yamcli-disk" => Some("disk"),
        "yamcli-daily-briefing" => Some("daily-briefing"),
        "yamcli-find-and-read" => Some("find-and-read"),
        "yamcli-reply-with-context" => Some("reply-with-context"),
        "yamcli-attachment-to-disk" => Some("attachment-to-disk"),
        "yamcli-send-file-by-mail" => Some("send-file-by-mail"),
        "yamcli-send-link-by-mail" => Some("send-link-by-mail"),
        "yamcli-publish-file-link" => Some("publish-file-link"),
        "yamcli-revoke-public-link" => Some("revoke-public-link"),
        "yamcli-invite-to-calendar" => Some("invite-to-calendar"),
        _ => None,
    }
}

pub fn prompt_skill_name(prompt_name: &str) -> Option<&'static str> {
    SKILLS.iter().find_map(|skill| {
        (skill_prompt_name(skill.name) == Some(prompt_name)).then_some(skill.name)
    })
}

/// Write all embedded skills to `target_dir/<skill-name>/SKILL.md`.
/// Returns the number of skills written.
pub fn install_skills(target_dir: &Path) -> Result<usize> {
    for skill in SKILLS {
        let skill_dir = target_dir.join(skill.name);
        fs::create_dir_all(&skill_dir).map_err(|err| {
            YamcliError::Io(format!(
                "failed to create skill directory {}: {err}",
                skill_dir.display()
            ))
        })?;
        let skill_path = skill_dir.join("SKILL.md");
        fs::write(&skill_path, skill.content).map_err(|err| {
            YamcliError::Io(format!(
                "failed to write skill {}: {err}",
                skill_path.display()
            ))
        })?;
    }
    Ok(SKILLS.len())
}

fn parse_frontmatter_description(content: &str) -> Option<String> {
    let frontmatter = content
        .strip_prefix("---\n")?
        .split_once("\n---\n")
        .map(|(frontmatter, _)| frontmatter)?;
    let raw = frontmatter
        .lines()
        .find_map(|line| line.strip_prefix("description:"))?
        .trim();
    Some(raw.trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn skill_count_matches_constant() {
        assert_eq!(SKILLS.len(), SKILL_COUNT);
    }

    #[test]
    fn skill_names_returns_all_names() {
        let names = skill_names();
        assert_eq!(names.len(), SKILL_COUNT);
        assert!(names.contains(&"yamcli-shared"));
        assert!(names.contains(&"yamcli-mail"));
        assert!(names.contains(&"yamcli-calendar"));
        assert!(names.contains(&"yamcli-disk"));
        assert!(names.contains(&"yamcli-daily-briefing"));
        assert!(names.contains(&"yamcli-find-and-read"));
        assert!(names.contains(&"yamcli-reply-with-context"));
        assert!(names.contains(&"yamcli-attachment-to-disk"));
        assert!(names.contains(&"yamcli-send-file-by-mail"));
        assert!(names.contains(&"yamcli-send-link-by-mail"));
        assert!(names.contains(&"yamcli-publish-file-link"));
        assert!(names.contains(&"yamcli-revoke-public-link"));
        assert!(names.contains(&"yamcli-invite-to-calendar"));
    }

    #[test]
    fn skill_names_are_valid_per_spec() {
        for skill in SKILLS {
            assert!(
                skill.name.len() <= 64,
                "{}: name exceeds 64 chars",
                skill.name
            );
            assert!(
                skill
                    .name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "{}: name contains invalid characters",
                skill.name
            );
            assert!(
                !skill.name.starts_with('-') && !skill.name.ends_with('-'),
                "{}: name starts or ends with hyphen",
                skill.name
            );
            assert!(
                !skill.name.contains("--"),
                "{}: name contains consecutive hyphens",
                skill.name
            );
        }
    }

    #[test]
    fn skill_content_has_valid_frontmatter() {
        for skill in SKILLS {
            assert!(
                skill.content.starts_with("---\n"),
                "{}: missing frontmatter start",
                skill.name
            );
            let after_first = &skill.content[4..];
            assert!(
                after_first.contains("\n---\n"),
                "{}: missing frontmatter end",
                skill.name
            );

            assert!(
                skill.content.contains(&format!("name: {}", skill.name)),
                "{}: frontmatter name does not match directory name",
                skill.name
            );
            assert!(
                skill.content.contains("description:"),
                "{}: missing description in frontmatter",
                skill.name
            );
        }
    }

    #[test]
    fn skill_content_within_size_limits() {
        for skill in SKILLS {
            let line_count = skill.content.lines().count();
            assert!(
                line_count <= 500,
                "{}: {} lines exceeds 500 line limit",
                skill.name,
                line_count
            );
        }
    }

    #[test]
    fn skill_description_within_spec_limit() {
        for skill in SKILLS {
            let fm_end = skill.content[4..].find("\n---\n").unwrap() + 4;
            let frontmatter = &skill.content[4..fm_end];
            let desc_line = frontmatter
                .lines()
                .find(|line| line.starts_with("description:"))
                .unwrap_or_else(|| panic!("{}: no description line", skill.name));
            let desc = desc_line.trim_start_matches("description:").trim();
            let desc = desc.trim_matches('"');
            assert!(
                desc.len() <= 1024,
                "{}: description is {} chars, exceeds 1024",
                skill.name,
                desc.len()
            );
            assert!(!desc.is_empty(), "{}: description is empty", skill.name);
        }
    }

    #[test]
    fn install_skills_writes_all_files() {
        let temp = tempdir().expect("tempdir");
        let count = install_skills(temp.path()).expect("install");
        assert_eq!(count, SKILL_COUNT);

        for skill in SKILLS {
            let path = temp.path().join(skill.name).join("SKILL.md");
            assert!(path.exists(), "{}/SKILL.md not found", skill.name);
            let content = fs::read_to_string(&path).expect("read");
            assert_eq!(content, skill.content);
        }
    }

    #[test]
    fn install_skills_is_idempotent() {
        let temp = tempdir().expect("tempdir");
        install_skills(temp.path()).expect("first install");
        let count = install_skills(temp.path()).expect("second install");
        assert_eq!(count, SKILL_COUNT);

        for skill in SKILLS {
            let path = temp.path().join(skill.name).join("SKILL.md");
            let content = fs::read_to_string(&path).expect("read");
            assert_eq!(content, skill.content);
        }
    }

    #[test]
    fn skill_content_and_description_are_available() {
        let content = skill_content("yamcli-mail").expect("skill content");
        assert!(content.contains("# yamcli mail"));

        let description = skill_description("yamcli-mail").expect("skill description");
        assert!(description.contains("Яндекс Почта"));
    }

    #[test]
    fn prompt_and_skill_name_mapping_is_bidirectional() {
        assert_eq!(prompt_skill_name("mail"), Some("yamcli-mail"));
        assert_eq!(
            prompt_skill_name("daily-briefing"),
            Some("yamcli-daily-briefing")
        );
        assert_eq!(skill_prompt_name("yamcli-shared"), Some("shared"));
        assert_eq!(
            skill_prompt_name("yamcli-reply-with-context"),
            Some("reply-with-context")
        );
        assert_eq!(
            prompt_skill_name("attachment-to-disk"),
            Some("yamcli-attachment-to-disk")
        );
        assert_eq!(
            skill_prompt_name("yamcli-send-file-by-mail"),
            Some("send-file-by-mail")
        );
        assert_eq!(
            prompt_skill_name("send-link-by-mail"),
            Some("yamcli-send-link-by-mail")
        );
        assert_eq!(
            prompt_skill_name("publish-file-link"),
            Some("yamcli-publish-file-link")
        );
        assert_eq!(
            prompt_skill_name("revoke-public-link"),
            Some("yamcli-revoke-public-link")
        );
        assert_eq!(
            prompt_skill_name("invite-to-calendar"),
            Some("yamcli-invite-to-calendar")
        );
        assert_eq!(
            skill_prompt_name("yamcli-publish-file-link"),
            Some("publish-file-link")
        );
        assert_eq!(
            skill_prompt_name("yamcli-revoke-public-link"),
            Some("revoke-public-link")
        );
        assert_eq!(
            skill_prompt_name("yamcli-invite-to-calendar"),
            Some("invite-to-calendar")
        );
    }
}
