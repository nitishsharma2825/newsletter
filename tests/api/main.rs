mod health_check;
mod helpers;
mod subscriptions;
mod subscriptions_confirm;

mod newsletter;

// structuring test as single test executable with scoped submodules for each test.
// Each submodule can be broken down further when it grows like tests/api/subscriptions/*.rs
