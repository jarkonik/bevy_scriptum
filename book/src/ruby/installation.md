# Installation

Ruby is currently only supported on Linux.

## Ruby

To build `bevy_scriptum` with Ruby support a Ruby installation is needed to be
present on your development machine.

The easiest way to produce a compatible Ruby installation is to use [rbenv](https://rbenv.org/).

After installing `rbenv` along with its `ruby-build` plugin you can build and
install a Ruby installation that will work with `bevy_scriptum` by executing:

```sh
CC=clang RUBY_CONFIGURE_OPTS="--enable-static --disable-shared" rbenv install 3.4.4
```

Before building make sure you are using the correct Ruby version.
It can be done for example by executing `which ruby`.
Output should be similar to `/home/$USER/.rbenv/shims/ruby`.
If the version is not correct then refer to `rbenv` manual or documentation
relevant for your method of installation to switch to correct Ruby.
To set Ruby version for current shell for example `rbenv shell 3.4.4` can be
used.

If ruby-static installation can't be found its advisable to run `cargo clean`
before building as `rb-sys` may have cached an incorrect Ruby version.

Above assumes that you also have `clang` installed on your system.
For `clang` installation instruction consult your
OS vendor provided documentation or [clang official webiste](https://clang.llvm.org).

If you rather not use `rbenv` you are free to supply your own installation of
Ruby provided the following is true about it:

- it is compiled with `clang`
- it is compiled as a static library
- it is accessible as `ruby` within `PATH` or `RUBY` environment variable is set
  to path of desired `ruby` binary.

## Main Library

Add the following to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.17"
bevy_scriptum = { version = "0.9", features = ["ruby"] }
```

If you need a different version of bevy you need to use a matching bevy_scriptum
version according to the [bevy support matrix](../bevy_support_matrix.md)

Ruby also needs dynamic symbol resolution and since `bevy_scriptum` links Ruby
statically the following `build.rs` file is needed to be present in project
root directory.

```rust
fn main() {
    println!("cargo:rustc-link-arg=-rdynamic");
}
```
