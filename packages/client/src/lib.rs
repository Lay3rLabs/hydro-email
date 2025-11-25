//! Abstractions for different backends (Climb, Climb Pool, MultiTest)
//! Provides AnyQuerier and AnyExecutor to represent _any_ contract querier/executor
//! The idea is that by moving the heavy-lifting here, we're free to write higher-level code
//! that provides an idiomatic and clean API
pub mod address;
pub mod contracts;
pub mod executor;
pub mod querier;
