
# Change Log


## Version 1.2.0 (20.03.2019)

- Added new `try_from_vec` which returns a `Result<Vec1<T>, Size0Error>`.
- Deprecated `from_vec` as it doesn't return a error type as error.

### Unstable/Nightly features

- New `unstable-nightly-try-from-impl` feature which adds a `TryFrom<Vec<T>>` implementation.


## Version 1.1.0

- Addead a `serde` feature implementing `Serialize`/`Deserialize`.
