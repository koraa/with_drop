# with_drop

Nostd wrapper that can be used to execute a custom destructor.

## Usage

Cargo.toml

```toml
[dependencies]
with_drop = "0.0.1"
```

In code:

```rust
use std::{cell::RefCell};
use with_drop::with_drop;

let drop_sum = RefCell::new(0);

{
  let mut v = with_drop(32, |x| { *drop_sum.borrow_mut() += x });

  // use value
  assert!(*v == 32);

  // Modify it
  *v = 42;
}

// Drop function should have been executed
assert!(*drop_sum.borrow() == 42);
```

## Motivation

Take the following bit of code:

```rust
use std::{io::Result, process::Command, process::Stdio};

fn main() -> Result<()> {
  let child1 = Command::new("echo").arg("42").stdout(Stdio::piped()).spawn()?;
  let child2 = Command::new("echo").arg("23").stdout(Stdio::piped()).spawn()?;
  assert!(child1.wait_with_output()?.stdout == b"42\n");
  assert!(child2.wait_with_output()?.stdout == b"23\n");
  Ok(())
}
```

Simple, right? We start two subprocesses, collect their results and compare them against a value.
Except, that this example is not entirely correct; it is not exception safe. The [std::process::Child](https://doc.rust-lang.org/std/process/struct.Child.html)
documentation specifies that [wait()](https://doc.rust-lang.org/std/process/struct.Child.html#method.wait)
must be called *manually* (Child does not implement Drop) to properly clean up behind the processes
(Failing to do so will result in [zombie processes](https://en.wikipedia.org/wiki/Zombie_process) under linux).

Now, if you take a closer look at the above code example, you might spot that not every code path does
call wait; if everything goes as planned, wait will be called, however if we exit early due to a failed
result, wait will not be called on either child1 or child2.

This property is called exception safety (or result safety since rust does not have exceptions?); the code example
above is not exception safe. We could manually use if statements to catch all these cases, but that would grow
very unwieldy very quickly. Optimally, the Child type would implement Drop and automatically wait on the processes.
but it doesn't. Failing that, we can use `with_drop()` to create a wrapper:

```rust
use std::{io::Result, process::Command, process::Stdio};
use with_drop::with_drop;

fn main() -> Result<()> {
  let child1 = with_drop(Command::new("echo").arg("42").stdout(Stdio::piped()).spawn()?, |mut child| {
    // Explicitly ignoring errors; the command might not have been started or might
    // not have ended or might have yielded an error; in any case we don't mind because
    // we just care about cleaning up zombies.
    let _ = child.wait();
  });
  let child2 = with_drop(Command::new("echo").arg("23").stdout(Stdio::piped()).spawn()?, |mut child| {
    let _ = child.wait();
  });
  assert!(child1.into_inner().wait_with_output()?.stdout == b"42\n");
  assert!(child2.into_inner().wait_with_output()?.stdout == b"23\n");
  Ok(())
}
```

### How about `finally()`

Couldn't a finally() like construction be used instead? No, it can not!
Our finally() guard would have to store a mutable reference of our child
variables which would prevent us from calling wait_with_output (borrow checker
complains).

```rust
use std::{io::Result, process::Command, process::Stdio};
use with_drop::with_drop;

struct Finally<F: FnMut()> {
  f: F
}

impl<F: FnMut()> Drop for Finally<F> {
    fn drop(&mut self) {
      (self.f)();
    }
}

fn finally<F: FnMut()>(f: F) -> Finally<F> {
  Finally { f }
}

fn main() -> Result<()> {
  let mut child1 = Command::new("echo").arg("42").stdout(Stdio::piped()).spawn()?;
  let finally_guard1 = finally(|| {
    let _ = child1.wait();
  });
  let mut child2 = Command::new("echo").arg("42").stdout(Stdio::piped()).spawn()?;
  let finally_guard2 = finally(|| {
    let _ = child2.wait();
  });

  // error[E0505]: cannot move out of `child1` because it is borrowed

  //assert!(child1.wait_with_output()?.stdout == b"42\n");
  //assert!(child2.wait_with_output()?.stdout == b"23\n");

  Ok(())
}
```

## Testing

Install clippy, rustfmt and nono:

```bash
$ rustup component add rustfmt
$ rustup component add clippy
$ RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-nono
```

And now use these to execute the tests.

```bash
$ cargo build
$ cargo test
$ cargo clippy --all-targets --all-features -- -D warnings
$ cargo fmt -- --check
$ cargo nono check
```

## License

Copyright © (C) 2020, Karolin Varner. All rights reserved.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
Neither the name of the Karolin Varner nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL Softwear, BV BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
