// platform/inventory — detectores de apps instaladas por fuente.

pub mod appx;
pub mod epic;
pub mod gog;
pub mod steam;
pub mod uninstall_registry;
pub mod winget;

pub use appx::{list_appx_packages, list_appx_provisioned};
pub use epic::list_epic_games;
pub use gog::list_gog_games;
pub use steam::list_steam_games;
pub use uninstall_registry::list_win32_apps;
pub use winget::list_winget_apps;
