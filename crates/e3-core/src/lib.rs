pub mod assignments;
pub mod auth;
pub mod calendar;
pub mod client;
pub mod courses;
pub mod error;
pub mod files;
pub mod forums;
pub mod grades;
pub mod ics;
pub mod notifications;
pub mod types;

#[cfg(test)]
mod tests;

pub use client::MoodleClient;
pub use error::E3Error;
