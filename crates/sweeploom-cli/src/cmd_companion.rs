//! `sweeploom companion-host` — browsers should launch `sweeploom-companion-host`.

use sweeploom_browser::run_native_host;
use sweeploom_platform::UserLocations;

pub fn run() {
    run_native_host(&UserLocations::current().app_data);
}
