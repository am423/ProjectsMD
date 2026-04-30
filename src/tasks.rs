//! Task parsing and manipulation.
//!
//! Handles parsing, creating, and updating task items from project.md task lists.
//! Task markers: `- [ ]` (pending), `- [x]` (done), `- [!]` (blocked).

use crate::sections::Section;

/// Task status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Done,
    Blocked,
}

/// A parsed task item from a project.md file.
#[derive(Debug, Clone)]
pub struct Task {
    pub status: TaskStatus,
    pub description: String,
    pub sub_items: Vec<String>,
    pub phase: String,
    pub number: Option<u32>,
    pub line_index: usize,
}

/// Check if a line is a task line.
fn is_task_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [!]")
}

/// Parse tasks from a section's content.
///
/// Recognizes `- [ ]`, `- [x]`, `- [!]` task markers.
/// Lines starting with `- ` that are not task markers are sub-items of the preceding task.
pub fn parse_tasks(content: &str, phase: &str) -> Vec<Task> {
    let mut tasks = Vec::new();
    let mut current_task: Option<Task> = None;
    let mut line_index: usize = 0;

    for line in content.lines() {
        if is_task_line(line) {
            // Save previous task
            if let Some(task) = current_task.take() {
                tasks.push(task);
            }

            let trimmed = line.trim_start();
            let status = if trimmed.starts_with("- [x]") {
                TaskStatus::Done
            } else if trimmed.starts_with("- [!]") {
                TaskStatus::Blocked
            } else {
                TaskStatus::Pending
            };

            let desc_start = trimmed.find(']').unwrap() + 1;
            let description = trimmed[desc_start..].trim().to_string();

            let (number, description) = parse_task_number(&description);

            current_task = Some(Task {
                status,
                description,
                sub_items: Vec::new(),
                phase: phase.to_string(),
                number,
                line_index,
            });
        } else if line.starts_with("  ") || line.starts_with("\t") {
            // Indented line — could be a sub-item
            if let Some(ref mut task) = current_task {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("- [") {
                    task.sub_items.push(trimmed.to_string());
                }
            }
        }
        // Non-indented, non-task lines are skipped
        line_index += 1;
    }

    // Save last task
    if let Some(task) = current_task {
        tasks.push(task);
    }

    tasks
}

/// Extract optional task number from description like "Task 4: Display formatting"
fn parse_task_number(description: &str) -> (Option<u32>, String) {
    if let Some(rest) = description.strip_prefix("Task ") {
        if let Some(colon_pos) = rest.find(':') {
            if let Ok(num) = rest[..colon_pos].trim().parse::<u32>() {
                let desc = rest[colon_pos + 1..].trim().to_string();
                return (Some(num), desc);
            }
        }
    }
    (None, description.to_string())
}

/// Parse tasks from all phase-related sections or subsections.
///
/// Looks for `### Phase: X` subsections inside the "Tasks" section.
/// Also handles top-level sections whose heading starts with "Phase:".
pub fn parse_all_tasks(sections: &[Section]) -> Vec<(String, Vec<Task>)> {
    let mut result = Vec::new();

    // First, check for a "Tasks" section containing ### Phase: subsections
    if let Some(tasks_section) = sections.iter().find(|s| s.heading == "Tasks") {
        let mut current_phase = String::new();
        let mut current_content = String::new();

        for line in tasks_section.content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("### Phase:") {
                // Save previous phase
                if !current_phase.is_empty() {
                    let tasks = parse_tasks(&current_content, &current_phase);
                    result.push((current_phase.clone(), tasks));
                }
                current_phase = rest.trim().to_string();
                current_content = String::new();
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }
        // Save last phase
        if !current_phase.is_empty() {
            let tasks = parse_tasks(&current_content, &current_phase);
            result.push((current_phase, tasks));
        }
    }

    // Also check for top-level sections whose heading starts with "Phase:"
    for section in sections.iter().filter(|s| s.heading.starts_with("Phase:")) {
        let phase = section
            .heading
            .strip_prefix("Phase:")
            .unwrap()
            .trim()
            .to_string();
        let tasks = parse_tasks(&section.content, &phase);
        result.push((phase, tasks));
    }

    result
}

/// Mark a task as done by its number.
/// Returns `true` if the task was found.
pub fn complete_task(tasks: &mut [Task], number: u32) -> bool {
    if let Some(task) = tasks.iter_mut().find(|t| t.number == Some(number)) {
        task.status = TaskStatus::Done;
        true
    } else {
        false
    }
}

/// Mark a task as blocked by its number.
/// Returns `true` if the task was found.
pub fn block_task(tasks: &mut [Task], number: u32, _reason: &str) -> bool {
    if let Some(task) = tasks.iter_mut().find(|t| t.number == Some(number)) {
        task.status = TaskStatus::Blocked;
        true
    } else {
        false
    }
}

/// Mark a blocked task as pending again.
/// Returns `true` if the task was found.
pub fn unblock_task(tasks: &mut [Task], number: u32) -> bool {
    if let Some(task) = tasks.iter_mut().find(|t| t.number == Some(number)) {
        if task.status == TaskStatus::Blocked {
            task.status = TaskStatus::Pending;
            return true;
        }
    }
    false
}

/// Append a new task line to the end of content.
pub fn add_task(content: &mut String, description: &str, phase: &str) {
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("- [ ] Task: {} (phase: {})\n", description, phase));
}

/// Format a task as a markdown line.
fn task_to_line(task: &Task) -> String {
    let marker = match task.status {
        TaskStatus::Pending => "- [ ]",
        TaskStatus::Done => "- [x]",
        TaskStatus::Blocked => "- [!]",
    };
    let desc = if let Some(num) = task.number {
        format!("Task {}: {}", num, task.description)
    } else {
        task.description.clone()
    };
    format!("{} {}", marker, desc)
}

/// Reconstruct a section's content from its tasks, preserving non-task lines.
pub fn write_tasks_to_section(section_content: &str, tasks: &[Task]) -> String {
    let mut result = String::new();
    let mut task_map: std::collections::HashMap<usize, &Task> = std::collections::HashMap::new();
    for task in tasks {
        task_map.insert(task.line_index, task);
    }

    let mut line_index: usize = 0;
    for line in section_content.lines() {
        if is_task_line(line) {
            if let Some(task) = task_map.get(&line_index) {
                result.push_str(&task_to_line(task));
                result.push('\n');
                // Skip sub-items from original; they're in the task
                line_index += 1;
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
        line_index += 1;
    }

    // Append any tasks that weren't in the original content
    for task in tasks {
        if task.line_index >= line_index {
            result.push_str(&task_to_line(task));
            result.push('\n');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const TASKS_CONTENT: &str = r#"### Phase: BUILD

- [x] Task 1: Project setup
- [x] Task 2: API client
- [x] Task 3: Caching layer
- [ ] Task 4: Display formatting
- [ ] Task 5: CLI argument parsing
"#;

    #[test]
    fn test_parse_tasks() {
        let tasks = parse_tasks(TASKS_CONTENT, "BUILD");
        assert_eq!(tasks.len(), 5);
        assert_eq!(tasks[0].status, TaskStatus::Done);
        assert_eq!(tasks[0].number, Some(1));
        assert_eq!(tasks[0].description, "Project setup");
        assert_eq!(tasks[3].status, TaskStatus::Pending);
        assert_eq!(tasks[3].number, Some(4));
        assert_eq!(tasks[3].description, "Display formatting");
    }

    #[test]
    fn test_parse_tasks_blocked() {
        let content = "- [!] Task 1: Blocked task\n";
        let tasks = parse_tasks(content, "BUILD");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::Blocked);
    }

    #[test]
    fn test_complete_task() {
        let mut tasks = parse_tasks(TASKS_CONTENT, "BUILD");
        assert!(complete_task(&mut tasks, 4));
        assert_eq!(tasks[3].status, TaskStatus::Done);
        assert!(!complete_task(&mut tasks, 99));
    }

    #[test]
    fn test_block_unblock_task() {
        let mut tasks = parse_tasks(TASKS_CONTENT, "BUILD");
        assert!(block_task(&mut tasks, 4, "waiting on API"));
        assert_eq!(tasks[3].status, TaskStatus::Blocked);
        assert!(unblock_task(&mut tasks, 4));
        assert_eq!(tasks[3].status, TaskStatus::Pending);
    }

    #[test]
    fn test_parse_all_tasks() {
        let content = std::fs::read_to_string(
            std::path::Path::new(env!("HOME")).join("projectsdotmd/example-cli-tool.md"),
        )
        .unwrap();
        let (_, body) = crate::frontmatter::parse_frontmatter(&content)
            .unwrap()
            .unwrap();
        let sections = crate::sections::parse_sections(body);
        let all_tasks = parse_all_tasks(&sections);
        assert_eq!(all_tasks.len(), 5); // DEFINE, DESIGN, BUILD, VERIFY, SHIP

        let build_phase = all_tasks.iter().find(|(name, _)| name == "BUILD");
        assert!(build_phase.is_some());
        let (_, build_tasks) = build_phase.unwrap();
        assert_eq!(build_tasks.len(), 7);
    }

    #[test]
    fn test_write_tasks_to_section() {
        let mut tasks = parse_tasks(TASKS_CONTENT, "BUILD");
        complete_task(&mut tasks, 4);
        let output = write_tasks_to_section(TASKS_CONTENT, &tasks);
        assert!(output.contains("- [x] Task 4: Display formatting"));
    }

    #[test]
    fn test_sub_items() {
        let content = "- [ ] Main task\n  - Sub item 1\n  - Sub item 2\n";
        let tasks = parse_tasks(content, "TEST");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].sub_items.len(), 2);
        assert_eq!(tasks[0].sub_items[0], "- Sub item 1");
    }
}
