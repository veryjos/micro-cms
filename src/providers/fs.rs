use std::{
    sync::{
        mpsc::channel,
        Arc,
        RwLock,
        RwLockReadGuard,
    },
    error::Error,
    fs::File,
    path::Path,
    str::FromStr,
    time::Duration,
    thread,
    panic,
};

use notify::{Watcher, RecursiveMode, watcher};
use walkdir::{WalkDir, DirEntry};

use crate::{
    entity::{Entity, FieldData},
    cache::Cache,
    schema::EntityDeclaration,
    providers::Provider,
    error::StringError,
};

pub struct RestartThread {
    join_handle: thread::JoinHandle<()>
}

impl RestartThread {
    pub fn new<F: (Fn()) + Send + panic::UnwindSafe + Clone + 'static>(func: F) -> RestartThread {
        let join_handle = thread::spawn(move || {
            loop {
                panic::catch_unwind(func.clone()).unwrap();
            }
        });

        RestartThread {
            join_handle
        }
    }

    pub fn join(self) {
        self.join_handle.join().unwrap();
    }
}

#[derive(Clone)]
pub struct FsProviderConfig {
    pub root: String,
}

pub struct FsProvider {
    cache: Arc<RwLock<Cache>>,
    restart_thread: RestartThread
}

fn decl_found(path: &Path, cache: &mut Cache) {
    let decl_name = path
        .file_name().unwrap()
        .to_str().unwrap();

    // Found a declaration, deserialize and send to cache
    let schema_path = path.join("schema");
    let contents = std::fs::read_to_string(schema_path).unwrap();
    let decl = EntityDeclaration::from_str(&contents).unwrap();

    cache.add_type(decl_name, decl);

    // Scan for entities associated with this declaration
    for ent_entry in WalkDir::new(path)
        .min_depth(1).max_depth(1).into_iter()
        .filter_entry(|e| !is_hidden(e) && e.file_type().is_file())
        .flatten()
        .filter(is_ent_file)
    {
        let path = ent_entry.path();

        // Found an entity, deserialize and send to cache
        let contents = std::fs::read_to_string(path).unwrap();
        let ent = Entity::from_str(&contents).unwrap();

        let ent_name = ent_entry.path()
            .file_stem().unwrap()
            .to_str().unwrap();

        cache.add_entity(decl_name, ent_name, ent);
    }

    // Search for folder representation of entities
    for ent_entry in WalkDir::new(path)
        .min_depth(1).max_depth(1).into_iter()
        .filter_entry(|e| !is_hidden(e) && e.file_type().is_dir())
        .flatten()
        .filter(is_ent_folder)
    {
        use std::ffi::OsStr;

        let path = ent_entry.path();

        // Found an entity, deserialize and send to cache
        let ent_path = path.join("ent");
        let contents = std::fs::read_to_string(&ent_path).unwrap();
        let mut ent = Entity::from_str(&contents).unwrap();

        // Search for any other fields
        for field_entry in WalkDir::new(path)
            .min_depth(1).max_depth(1).into_iter()
            .filter_entry(|e| !is_hidden(e) && e.file_type().is_file())
            .flatten()
            .filter(|e| e.file_name() != OsStr::new("ent"))
        {
            let field_name = field_entry.path()
                .file_stem().unwrap()
                .to_str().unwrap();

            match std::fs::read_to_string(field_entry.path()) {
                Ok(contents) => {
                    ent.fields.insert(field_name.to_owned(), FieldData::Str(contents));
                },

                _ => {
                    use std::io::Read;

                    let mut file = File::open(field_entry.path()).unwrap();
                    let mut data = Vec::new();
                    file.read_to_end(&mut data).unwrap();
                    ent.fields.insert(field_name.to_owned(), FieldData::Bin(data));
                }
            }

        }

        // Add the entity
        let ent_name = ent_entry.path()
            .file_name().unwrap()
            .to_str().unwrap();

        cache.add_entity(decl_name, ent_name, ent);
    }
}

fn create_cache(base_path: &Path) -> Cache {
    // Create a new cache
    let mut cache = Cache::new();

    // Populate declarations by traversing the filesystem
    for folder_entry in WalkDir::new(base_path)
        .min_depth(1).into_iter()
        .filter_entry(|e| !is_hidden(e) && e.file_type().is_dir())
        .flatten()
        .filter(is_decl_folder)
    {
       decl_found(folder_entry.path(), &mut cache);
    }

    cache
}

impl FsProvider {
    pub fn new(config: FsProviderConfig) -> FsProvider {
        // Convert the relative path in config to an absolute path
        let base_path = Path::new(&config.root).canonicalize().unwrap();

        // Create an initial cache
        let cache_lock = {
            let cache = create_cache(&base_path).validated();

            Arc::new(RwLock::new(cache))
        };

        let update_cache = {
            let cache_lock = cache_lock.clone();

            move |new_cache: Cache| {
                let mut guard = cache_lock.write().unwrap();

                *guard = new_cache;
            }
        };

        // Watch the filesystem to update the cache on modification
        let restart_thread = {
            let base_path = base_path.clone();

            RestartThread::new(move || {
                let (tx, rx) = channel();

                let mut watcher = watcher(tx, Duration::from_millis(1000)).unwrap();
                watcher.watch(base_path.clone(), RecursiveMode::Recursive).unwrap();

                while let Ok(event) = rx.recv() {
                    use notify::DebouncedEvent::*;

                    match event {
                        Write(_) | Create(_) |
                        Remove(_) | Rename(_, _) |
                        Rescan => update_cache(create_cache(&base_path).validated()),
                        
                        _ => {}
                    };
                }
            })
        };

        FsProvider {
            cache: cache_lock.clone(),
            restart_thread,
        }
    }
}

impl Provider for FsProvider {
    fn read_cache(&self) -> Result<RwLockReadGuard<Cache>, Box<dyn Error>> {
        self.cache.read().map_err(
            |_| Box::new(StringError::new("Failed to acquire read-lock on cache")) as Box<dyn Error>
        )
    }

    fn join(self) {
        self.restart_thread.join()
    }
}

fn is_decl_folder(entry: &DirEntry) -> bool {
    // Are we even a folder?
    if !entry.file_type().is_dir() {
        return false;
    }

    // Look for a schema file in the root of the folder
    entry.path()
        .join(Path::new("schema"))
        .exists()
}

fn is_ent_file(entry: &DirEntry) -> bool {
    use std::ffi::OsStr;

    // Check for extension
    entry.path().extension()
        .map_or(false, |e| e == OsStr::new("ent"))
}

fn is_ent_folder(entry: &DirEntry) -> bool {
    let ent_path = entry.path().join("ent");
    ent_path.exists()
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
