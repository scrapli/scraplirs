extern crate scraplirs;

use env_logger::{
    Builder,
    Target,
};
use log::LevelFilter;
use scraplirs::driver::{
    GenericDriver,
    GenericDriverBuilder,
};
use std::env;

// obviously set these to whatever you want to test with!
const ENABLE_LOGGING: bool = false;
const HOST: &str = "XYZ";
const USER: &str = "XYZ";
const PASSWORD: &str = "XYZ";
const DEV_NULL: &str = "/dev/null";
const COMMAND: &str = "show version | i Version";
const COMMAND_COUNT: i64 = 5;

/// Enable (or not) some logging for our example.
fn enable_logging() {
    if !ENABLE_LOGGING {
        return;
    }

    env::set_var("RUST_LOG", "TRACE");

    let mut builder = Builder::from_default_env();

    builder.target(Target::Stdout);
    builder.filter_level(LevelFilter::Trace);

    env_logger::init();
}

/// Build and return the generic driver object. Note that the builder can be chained without having
/// to re-assign but this is broken up more just for (hopefully) clarity.
fn setup_connection() -> GenericDriver {
    let mut driver_builder = GenericDriverBuilder::new(HOST.to_string());

    // user/password, obviously can skip if you're authing w/ a key/key from ssh config file
    driver_builder = driver_builder
        .user(USER.to_string())
        .password(PASSWORD.to_string());

    // we'll disable strict key checking too
    driver_builder = driver_builder.ssh_strict_key(false);

    // you can either set ssh config file path or in our case just point to /dev/null
    driver_builder = driver_builder.ssh_config_file_path(DEV_NULL.to_string());

    // depending on your device and if you use a config file with this already set or not, you may
    // need to pass some key type/kex/key algos...
    driver_builder = driver_builder.system_extra_args(vec![
        String::from("-o"),
        String::from("PubkeyAcceptedKeyTypes=+ssh-rsa"),
        String::from("-o"),
        String::from(
            "KexAlgorithms=+diffie-hellman-group-exchange-sha1,diffie-hellman-group14-sha1",
        ),
        String::from("-o"),
        String::from(
            "HostKeyAlgorithms=+ssh-dss,ssh-rsa,rsa-sha2-512,rsa-sha2-256,ssh-rsa,ssh-ed25519",
        ),
    ]);

    driver_builder.build()
}

/// Open a connection with a generic driver, fetch the prompt, and send some command N times. Print
/// out the average time per send operation.
fn main() {
    enable_logging();

    let mut driver = setup_connection();

    driver.open().expect("failed opening connection");

    let prompt = driver
        .channel
        .get_prompt()
        .expect("failed finding device prompt");

    println!(
        "found device prompt: {}",
        std::str::from_utf8(&prompt).expect("failed decoding prompt")
    );

    let mut total_milliseconds: i64 = 0;

    for i in 0..COMMAND_COUNT {
        println!("sending command '{}' for the {} time", COMMAND, i);

        let resp = driver
            .send_command(COMMAND)
            .expect("failed sending command");

        total_milliseconds += resp.elapsed_time.num_milliseconds();

        println!("result:\n{}\n", resp.result);
    }

    println!(
        "\nsending command '{}' {} times took on average {} milliseconds",
        COMMAND,
        COMMAND_COUNT,
        total_milliseconds / COMMAND_COUNT
    );
}
