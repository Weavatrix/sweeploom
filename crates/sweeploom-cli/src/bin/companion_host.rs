//! Dedicated native-messaging host. Browsers launch this with no extra args.

fn main() {
    sweeploom_browser::run_native_host(&sweeploom_platform::UserLocations::current().app_data);
}
