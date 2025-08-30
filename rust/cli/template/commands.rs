use devalang_utils::file::format_file_size;
use include_dir::{Dir, DirEntry, include_dir};

static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[cfg(feature = "cli")]
pub fn handle_template_list_command() {
    let available_templates = get_available_templates();

    println!("📦 Available templates ({}) :\n", available_templates.len());

    for dir in available_templates {
        println!("• {}", dir);
    }

    println!("\nUsage : devalang init --name <project-name> --template <template-name>");
}

#[cfg(feature = "cli")]
pub fn handle_template_info_command(name: String) {
    let template_dir = TEMPLATES_DIR.get_dir(name.clone()).unwrap_or_else(|| {
        println!("❌ The template '{}' is not found.", name);

        std::process::exit(1);
    });

    let mut file_count = 0;
    let mut dir_count = 0;
    let mut total_size: u64 = 0;

    fn walk(dir: &Dir, file_count: &mut u32, dir_count: &mut u32, total_size: &mut u64) {
        for entry in dir.entries() {
            match entry {
                DirEntry::File(file) => {
                    *file_count += 1;
                    *total_size += file.contents().len() as u64;
                }
                DirEntry::Dir(subdir) => {
                    *dir_count += 1;
                    walk(subdir, file_count, dir_count, total_size);
                }
            }
        }
    }

    walk(
        template_dir,
        &mut file_count,
        &mut dir_count,
        &mut total_size,
    );

    println!("📦 Template : {}", name);
    println!("📂 Content  : {file_count} file(s), {dir_count} folder(s)");
    println!("💾 Size     : {}", format_file_size(total_size));
}

pub fn get_available_templates() -> Vec<String> {
    TEMPLATES_DIR
        .dirs()
        .map(|dir| {
            dir.path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .collect()
}
