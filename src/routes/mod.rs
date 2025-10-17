// declare submodules
mod health_check;
mod newsletters;
mod subscriptions;
mod subscriptions_confirm;

// re-export public items from submodules to make them accessible from outside
pub use health_check::*;
pub use newsletters::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
