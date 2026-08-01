use std::process::Command;

pub trait GitInfo {
    fn describe(&self) -> Option<String>;
}

pub struct GitCommand {
    pub working_dir: String,
}

impl GitInfo for GitCommand {
    fn describe(&self) -> Option<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.working_dir)
            .arg("status")
            .arg("--short")
            .arg("--branch")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8(output.stdout).ok()?;
        let mut lines = text.lines();
        let branch_line = lines.next()?.trim_start_matches("## ");
        let is_dirty = lines.next().is_some();

        Some(match is_dirty {
            true => format!("{} ●", branch_line),
            false => branch_line.to_string(),
        })
    }
}
