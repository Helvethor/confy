use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::ffi::OsString;
use std::error::Error;

use inotify::{Event, WatchMask, WatchDescriptor, Inotify};
use ::PathBinding;
use generator::Generator;
use config::{Config, ConfigFiles};


pub struct Watcher<'a> {
    config_files: ConfigFiles<'a>,
    config_wd: ConfigWatchDescriptors,
    inotify: Inotify,
    generator: Option<Generator>,
    watches: HashMap<WatchDescriptor, HashMap<OsString, PathBinding>>,
    mode: Mode
}

struct ConfigWatchDescriptors {
    bindings: WatchDescriptor,
    variables: WatchDescriptor
}

pub struct Mode {
    pub files: bool,
    pub bindings: bool,
    pub variables: bool
}


impl<'a> Watcher<'a> {
    pub fn new(config_files: ConfigFiles<'a>, mode: Mode) -> Result<Watcher, String> {
        let mut inotify = match Inotify::init() {
            Ok(i) => i,
            Err(e) => return Err(format!("Couldn't open inotify: {}", e))
        };

        let bindings_wd = match Watcher::get_file_wd(&mut inotify, config_files.bindings) {
            Ok(wd) => wd,
            Err(e) => return Err(e)
        };
        let variables_wd = match Watcher::get_file_wd(&mut inotify, config_files.variables) {
            Ok(wd) => wd,
            Err(e) => return Err(e)
        };
        let config_wd = ConfigWatchDescriptors {
            bindings: bindings_wd,
            variables: variables_wd
        };

        let mut watcher = Watcher {
            config_files,
            config_wd,
            inotify,
            generator: None,
            watches: HashMap::new(),
            mode
        };

        match watcher.update() {
            Ok(_) => Ok(watcher),
            Err(e) => Err(e),
        }
    }

    fn get_file_wd(inotify: &mut Inotify, file: &Path) -> Result<WatchDescriptor, String> {
        match file.parent() {
            Some(p) => match inotify.add_watch(p, WatchMask::CLOSE_WRITE) {
                Ok(wd) => Ok(wd),
                Err(e) => Err(format!(
                    "Could add directory watch: {}", e.description()))
            },
            None => Err(format!(
                "Couldn't get file's directory: {}", file.display()))
        }
    }



    fn update(&mut self) -> Result<(), String> {

        let mut config = match Config::new(&self.config_files) {
            Ok(c) => c,
            Err(e) => return Err(e)
        };
        let generator = Generator::new(&config.variables);
        let mut watches = HashMap::new();

        while let Some(binding) = config.bindings.pop() {
            let dir = binding.from.parent().unwrap().to_owned();

            match self.inotify.add_watch(dir, WatchMask::CLOSE_WRITE) {
                Ok(wd) => {
                    let file_name = binding.from.file_name().unwrap().to_owned();
                    if watches.contains_key(&wd) {
                        let map: &mut HashMap<OsString, PathBinding>
                            = watches.get_mut(&wd).expect(
                            "Missing HashMap for a WatchDescriptor");
                        map.insert(file_name, binding);
                    }
                    else {
                        let mut map = HashMap::new();
                        map.insert(file_name, binding);
                        watches.insert(wd, map);
                    }
                },
                Err(e) => return Err(format!("Couldn't add inotify watch: {}", e))
            };
        }

        for (wd, _) in self.watches.drain() {
            if !watches.contains_key(&wd) {
                match self.inotify.rm_watch(wd) {
                    Ok(_) => (),
                    Err(e) => warn!("Couldn't remove inotify watch: {}", e)
                }
            }
        }

        self.watches = watches;
        self.generator = Some(generator);

        debug!("{:?}", self.watches);

        Ok(())
    }

    pub fn watch(&mut self) -> ! {
        let mut buffer = [0u8; 4096];
        self.process_all();

        loop {
            let events = match self.inotify.read_events_blocking(&mut buffer) {
                Ok(e) => e,
                Err(e) => panic!("Couldn't read inotify events: {}", e)
            };

            for event in events {
                debug!("Event: {:?}", event);
                if self.handle_event(event) {
                    break;
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> bool {
        if (self.mode.variables || self.mode.bindings)
            && self.is_config_event(&event) {
            match self.update() {
                Ok(()) => {
                    info!("internal configuration updated");
                    self.process_all();
                    return true;
                },
                Err(e) => error!("{}", e)
            };
        }
        if self.mode.files {
            if let Some(binding) = self.get_binding(&event) {
                debug!("{:?}", binding);
                self.process(binding);
            }
        }
        false
    }

    fn process(&self, binding: &PathBinding) {
        if let Some(ref generator) = self.generator {
            match generator.process(binding) {
                Ok(n) => info!("{}: replaced {} key(s)", binding.from.display(), n),
                Err(e) => error!("{}", e)
            };
        }
        else {
            error!("Inconsistent internal state: missing Some(generator)");
        }
    }

    fn process_all(&self) {
        for (_, map) in self.watches.iter() {
            for (_, binding) in map.iter() {
                self.process(binding);
            }
        }
    }

    fn get_binding(&self, event: &Event) -> Option<&PathBinding> {
        if let Some(file_name) = event.name {
            if let Some(map) = self.watches.get(&event.wd) {
                if let Some(binding) = map.get(file_name) {
                    return Some(binding);
                }
            }       
        }
        None
    }

    fn is_config_event(&self, event: &Event) -> bool {
        return self.is_config_file_event(event, self.config_files.bindings, &self.config_wd.bindings)
            || self.is_config_file_event(event, self.config_files.variables, &self.config_wd.variables);
    }

    fn is_config_file_event(&self, event: &Event, config_file: &Path, config_wd: &WatchDescriptor) -> bool {
        if let Some(name) = event.name {
            if &event.wd == config_wd {
                let mut current = PathBuf::from(config_file);
                while let Ok(next) = current.read_link() {
                    current = next;
                }
                if name == current.file_name().unwrap() {
                    return true;
                }
            }
        }
        false
    }
}
