use {
    std::{fmt, sync::Mutex, thread, time::Duration},
    swayipc::{Connection, Event, EventType, InputChange},
};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
    }
}

fn run() -> Result<(), Error> {
    let mut conn = Connection::new()?;
    let inputs = conn.get_inputs()?;
    let input = inputs
        .into_iter()
        .find(|input| input.xkb_active_layout_name.is_some())
        .ok_or(Error::InputNotFound)?;

    let current_input = input.identifier;
    let current_layout = input
        .xkb_active_layout_name
        .map(Mutex::new)
        .unwrap_or_default();

    thread::scope(|s| {
        s.spawn(|| loop {
            thread::sleep(Duration::from_secs(1));
            let current_layout = current_layout.lock().expect("lock");
            layout_event(&current_layout);
        });

        let subs = [EventType::Input];
        for event in conn.subscribe(subs)? {
            let Ok(Event::Input(inp))  = event else { continue };
            let InputChange::XkbLayout = inp.change else { continue };
            let input = inp.input;
            if input.identifier == current_input {
                let new_layout = input.xkb_active_layout_name.unwrap_or_default();
                let mut current_layout = current_layout.lock().expect("lock");
                *current_layout = new_layout;
                layout_event(&current_layout);
            }
        }

        panic!("the event loop should be infinite")
    })
}

fn layout_event(layout: &str) {
    println!("layout: {layout}");
}

enum Error {
    Sway(swayipc::Error),
    InputNotFound,
}

impl From<swayipc::Error> for Error {
    fn from(v: swayipc::Error) -> Self {
        Self::Sway(v)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Sway(err) => write!(f, "{err}"),
            Self::InputNotFound => write!(f, "input not found"),
        }
    }
}
