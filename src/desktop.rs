use std::process::Command;

pub fn check_desktop() -> Desktop{
    let output = Command::new("pgrep").arg("-l").arg("gnome|kde").output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    return if let Some(_) = output.find("gnome-") {
        Desktop::Gnome
    } else if let Some(_) = output.find("kde-") {
        Desktop::KDE
    } else {
        Desktop::Others
    }
}

pub enum Desktop {
    Gnome,
    KDE,
    // Mate,
    // LXde,
    // Cinnamon,
    // Xfce,
    // Jwm,
    Others,
}

pub fn terminal_prefix(desktop: Desktop) -> (&'static str, Vec<&'static str>) {
    match desktop {
        Desktop::Gnome => {("gnome-terminal",vec!["--wait", "--"])}
        Desktop::KDE => {("konsole",vec!["--wait", "--"])}
        Desktop::Others => {("bash",vec!["-c"])}
    }
}