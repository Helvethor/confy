use std::path::PathBuf;
use std::collections::HashMap;
use std::ffi::OsString;

use inotify::{Event, WatchMask, WatchDescriptor, Inotify};
use ::PathBinding;
use generator::Generator;
use config::{Config, ConfigFiles};


pub struct Watcher<'a> {
    config_files: ConfigFiles<'a>,
    inotify: Inotify,
    generator: Option<Generator>,
    watches: Watches,
    mode: Mode
}

type Watches = HashMap<ElementDescriptor, WatchedElement>;

#[derive(Debug)]
enum WatchedElement {
    Binding(PathBinding),
    Config(PathBuf)
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct ElementDescriptor {
    wd: WatchDescriptor,
    file_name: OsString
}

pub struct Mode {
    pub files: bool,
    pub bindings: bool,
    pub variables: bool
}


impl<'a> Watcher<'a> {
    pub fn new(config_files: ConfigFiles<'a>, mode: Mode) -> Result<Watcher, String> {
        let inotify = match Inotify::init() {
            Ok(i) => i,
            Err(e) => return Err(format!("Couldn't open inotify: {}", e))
        };

        let mut watcher = Watcher {
            config_files,
            inotify,
            generator: None,
            watches: Watches::new(),
            mode
        };

        match watcher.update() {
            Ok(_) => Ok(watcher),
            Err(e) => Err(e),
        }
    }

    fn add_watch(&mut self, watches: &mut Watches, element: WatchedElement)
        -> Result<(), String>
    {
        let dir = element.target().parent().unwrap().to_owned();
        debug!("dir {}", dir.display());
        match self.inotify.add_watch(dir, WatchMask::CLOSE_WRITE) {
            Ok(wd) => {
                let file_name = element.target().file_name().unwrap().to_owned();
                let descriptor = ElementDescriptor {
                    wd,
                    file_name
                };
                watches.insert(descriptor, element);
            },
            Err(e) => return Err(format!("Couldn't add inotify watch: {}", e))
        };
        Ok(())
    }

    fn update(&mut self) -> Result<(), String> {

        let mut config = match Config::new(&self.config_files) {
            Ok(c) => c,
            Err(e) => return Err(e)
        };
        let generator = Generator::new(&config.variables);
        let mut watches = Watches::new();
        let mut elements = Vec::new();

        if self.mode.files {
            while let Some(binding) = config.bindings.pop() {
                let element = WatchedElement::Binding(binding);
                elements.push(element);
            }
        }
        
        if self.mode.bindings {
            let element = WatchedElement::Config(
                self.config_files.bindings.to_owned());
            elements.push(element);
        }

        if self.mode.variables {
            let element = WatchedElement::Config(
                self.config_files.variables.to_owned());
            elements.push(element);
        }

        while let Some(element) = elements.pop() {
            if let Err(e) = self.add_watch(&mut watches, element) {
                return Err(e);
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
        if let Some(file_name) = event.name {
            let mut update = false;
            let descriptor = ElementDescriptor {
                wd: event.wd,
                file_name: file_name.to_owned()
            };

            if let Some(element) = self.watches.get(&descriptor) {
                match element {
                    &WatchedElement::Binding(ref binding) => self.process(binding),
                    &WatchedElement::Config(_) => update = true
                };
            };

            if update {
                match self.update() {
                    Ok(()) => {
                        info!("internal configuration updated");
                        self.process_all();
                        return true;
                    },
                    Err(e) => error!("{}", e)
                }
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
        for watched_element in self.watches.values() {
            if let &WatchedElement::Binding(ref binding) = watched_element {
                self.process(binding);
            }
        }
    }
}


impl WatchedElement {

    fn target(&self) -> PathBuf {
        match self {
            &WatchedElement::Binding(ref binding) => binding.from.as_path(),
            &WatchedElement::Config(ref config_file) => config_file.as_path()
        }.canonicalize().unwrap()
    }
}
