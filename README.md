# XRadar

Scan a host for open ports.

## Demo

<a title="Click to view ASCII" href="https://asciinema.org/a/495524?autoplay=1">
  <img width=40% src="https://github.com/miraclx/xradar/raw/master/media/demo.gif" alt="ASCII Demo">
</a>

## Installation

First, install Rust and Cargo. See <https://rustup.rs/>.

```bash
cargo install xradar
```

## Usage

```text
xr [host] [port...] [options...]
```

`[port...]` can be a comma-separated or space-separated list of ports.

See full help information with the `--help` flag.

## Examples

- Scan all open ports on `localhost`:

  ```console
  xr
  ```

- Scan all open ports on `192.168.0.200`:
  
  ```console
  xr 192.168.0.200
  ```

- Scan open ports between `22` and `80` and greater than `1024` on `1.1.1.1`:
  
  ```console
  xr 1.1.1.1 22..80 1024..
  ```

  Alternatively, you can use the `22-80,1024-` syntax.

- Check the status of `80` and `443` on `216.58.223.206`:

  ```console
  xr 216.58.223.206 80,443 -a
  ```

  The `-a` flag will show the status of ports, even when they are not open.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as below, without any additional terms or conditions.

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
