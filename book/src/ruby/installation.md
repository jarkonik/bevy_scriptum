# Installation

## Ruby

To build `bevy_scriptum` with Ruby support a Ruby installation is needed to be
present on your development machine.

The easiest way to produce a compatible Ruby installation is to use [rbenv](https://rbenv.org/).

After installing `rbenv` along with its `ruby-build` plugin you can build and
install a Ruby installation that will work with `bevy_scriptum` by executing:

```sh
CC=clang rbenv install 3.4.4
```

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
bevy = "0.16"
bevy_scriptum = { version = "0.8", features = ["ruby"] }
```

If you need a different version of bevy you need to use a matching bevy_scriptum
version according to the [bevy support matrix](../bevy_support_matrix.md)
