use notify::{ Watcher, RecursiveMode, Config, RecommendedWatcher };
use std::sync::mpsc::channel;

pub fn watch_directory<F>(entry: String, callback: F) -> notify::Result<()>
    where F: Fn() + Send + 'static
{
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    watcher.watch(&entry.as_ref(), RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(_) => {
                callback();
            }
            Err(e) => {
                eprintln!("Channel error : {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
