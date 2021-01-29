
# Change Log

## Version 1.7.0 (pending)

- minimal rust version is now 1.47
- (pending) support for smallvec (v>=1.6.1)

## Version 1.6.0 (11.08.2020)

- Added the `split_off_first` and `split_off_last` methods.

## Version 1.5.1 (01.07.2020)

- Updated project to `edition="2018"` (not that this is
  a purely internal change and doesn't affect the API
  interface or minimal supported rustc version)
- Added [CONTRIBUTORS.md](./CONTRIBUTORS.md)
- Updated [README.md](./README.md)

## Version 1.5.0 (21.05.2020)

- minimal rust version is now 1.34
- `TryFrom` is no longer feature gated
- `vec1![]` now allows trailing `,` in all cases
- `Size0Error` now no longer has a custom
  `std::error::Error::description()` implementation.
- fixed various clippy::pedantic warnings
- updated `Cargo.toml`
- `cargo fmt`

## Version 1.4.0 (26.03.2019)

New trait impl:
- impl Default for Vec1<T> where T: Default

## Version 1.3.0 (21.03.2019)

New manual proxied methods:
- splice
- to_asci_lowercase
- to_asci_upercase

New Into impl for following types:
- Rc<[T]>
- Arc<[T]>
- Box<[T]>
- VecDeque<T>

### Unstable/Nightly features

New TryFrom impl for following types:
- Box<[T]>
- BinaryHeap<T>
- VecDeque<T>
- String
- &str
- &[T] where T: Clone
- &mut [T] where T: Clone

## Version 1.2.0 (20.03.2019)

- Added new `try_from_vec` which returns a `Result<Vec1<T>, Size0Error>`.
- Deprecated `from_vec` as it doesn't return a error type as error.

### Unstable/Nightly features

- New `unstable-nightly-try-from-impl` feature which adds a `TryFrom<Vec<T>>` implementation.


## Version 1.1.0

- Addead a `serde` feature implementing `Serialize`/`Deserialize`.
