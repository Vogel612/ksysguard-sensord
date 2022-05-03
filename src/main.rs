extern crate ksysguard_sensord as this;

mod main {
    use clap::Parser;
    use std::path::Path;

    use this::config::Config;

    /// A ksysguardd extension supporting configurable arbitrary sensor data
    /// obtained by invoking external commands. Supports both polling command
    /// uses as well as streaming commands.
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about = None)]
    struct Args {
        #[clap(short, long)]
        port: Option<u16>,

        #[clap(long, default_value = &"/etc/ksysguard-sensord/config")]
        config: String,

        #[clap(short, long)]
        daemon: bool,
    }

    pub fn main() {
        // read configuration
        let args = Args::parse();
        let cfg = Config::read_config(&Path::new(&args.config));

        if args.daemon {
            std::thread::spawn(move || {
                run(args, cfg);
            });
        } else {
            run(args, cfg);
        }
    }

    fn run(args: Args, mut config: Config) {
        println!("Has been given Args: {:?}", args);
        println!("Discovered config: {:?}", config);

        // merge args into config where necessary
        match args.port {
            Some(port) => config.port = port,
            None => {}
        }
        this::server::Sensord::start(&config);
    }
}

fn main() {
    main::main();
}
