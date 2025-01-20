use std::env;
use fxhash::{FxHashMap, FxBuildHasher};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input>", args[0]);
        std::process::exit(1);
    }

    let raw = std::fs::read_to_string(&args[1]).unwrap();


    let stations: FxHashMap<String, Station> = process_raw_stations(&raw);
    print_stations(&stations);
}


struct Station {
    min: f64,
    max: f64,
    accumulated: f64,
    count: usize,
}

impl Station {
    fn new(value: f64) -> Self {
        Self {
            min: value,
            max: value,
            accumulated: 0.0,
            count: 0,
        }
    }

    fn add_measurement(&mut self, value: f64) {
        self.accumulated += value;
        self.count += 1;

        if value > self.max {
            self.max = value;
        }
        if value < self.min {
            self.min = value;
        }
    }

    fn average(&self) -> f64 {
        self.accumulated / self.count as f64
    }
}

impl std::fmt::Display for Station {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:.1}/{:.1}/{:.1}",
            self.min,
            self.average(),
            self.max,
        )
    }
}

fn process_raw_stations(input: &str) -> FxHashMap<String, Station> {
    let mut stations = FxHashMap::with_hasher(FxBuildHasher::default());

    for line in input.lines() {
        let mut parts = line.split(|c| c == ';');

        let name = parts.next().unwrap();
        let value = parts.next().unwrap().parse::<f64>().unwrap();

        let station_maybe = stations.get_mut(name);

        let station = if station_maybe.is_some() {
            station_maybe.unwrap()
        } else {
            drop(station_maybe);
            stations.insert(name.to_string(), Station::new(value));
            stations.get_mut(name).unwrap()
        };

        station.add_measurement(value);
    }

    stations
}

fn print_stations(stations: &FxHashMap<String, Station>) {
    print!("{{");

    // Need to print by name in alphabetical order.
    let mut sorted_stations: Vec<(&String, &Station)> = stations.iter().collect();
    sorted_stations.sort_by(|a, b| a.0.cmp(b.0));

    let mut station_iter = sorted_stations.iter();

    // First station doesn't have a comma before it so do it separately
    let first = station_iter.next().unwrap();
    print!("{}={}", first.0, first.1);

    for (name, station) in station_iter {
        print!(", {}={}", name, station);
    }
    print!("}}\n");
}
