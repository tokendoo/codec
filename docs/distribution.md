# Node and Python distribution

The free Apache-2.0 codec uses the same Rust implementation in Node (WebAssembly)
and Python (native extension). The wire format is unchanged. No network calls,
request logging or telemetry are introduced by the bindings.

## Build and verify

```sh
./crates/tokendoo-wasm/build-npm.sh
maturin build --release -m crates/tokendoo-py/Cargo.toml
python scripts/test-wheel.py
```

The Node build requires Rust with `wasm32-unknown-unknown`, wasm-pack, Node 22 and
Python 3.11 or newer. The Python build requires maturin 1.7–1.x and Rust. Rust release optimization is
used; the extra Binaryen optimization pass is disabled because older builds can
cap wasm-bindgen’s exported externref table and break Node initialization
([upstream diagnosis](https://github.com/wasm-bindgen/wasm-bindgen/issues/4528)).
`bindings.yml` tests isolated installs on Linux, macOS and Windows, plus all nine
Rust/Node/Python encode/decode combinations. Linux ARM64 is cross-built; its wheel
is not executed on the x86 runner. Python wheels use CPython's stable ABI (3.9+).

`release-bindings.yml` creates the npm tarball, four platform wheels and a source
archive. It tests the *downloaded release artifacts* before a run is eligible for
publication. Run this workflow on `main` or a `v*` tag. The separate manual
`publish-bindings.yml` accepts the successful release run ID and `npm`, `pypi` or
`both`. Run it on the same commit. It publishes those artifacts without rebuilding.

## Registry access (one-time account setup)

GitHub push access does not grant npm or PyPI ownership. No publication token is
stored in this repository. Configure trusted publishing with these exact fields:

| Field | npm | PyPI |
| --- | --- | --- |
| Package / project | `tokendoo` | `tokendoo` |
| GitHub owner | `tokendoo` | `tokendoo` |
| Repository | `codec` | `codec` |
| Workflow filename | `publish-bindings.yml` | `publish-bindings.yml` |
| Environment | `npm` | `pypi` |

PyPI can use a **pending publisher** to create this new project on its first
publication. Configure it in the account Publishing page. npm trusted publisher
settings are attached to an existing package: for a new package, an authorized
npm account must bootstrap the verified tarball with `npm login` and
`npm publish /absolute/path/tokendoo-0.1.0.tgz --access public`, then configure the
trusted publisher in package settings. Do not paste account tokens into issues,
chat, logs or repository files. npm's OIDC client requires npm 11.5.1+ and Node
22.14.0+; the workflow installs npm 11 on Node 22.

Official setup guides: [npm](https://docs.npmjs.com/trusted-publishers/) and
[PyPI pending publishers](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/).

## Verify after publication

Use fresh directories / virtual environments to install `npm install tokendoo`
and `pip install tokendoo`, then run the committed package tests against those
installed packages. Confirm both registry versions match the release version
before changing the landing page from the distribution waitlist to install links.
The waitlist only authorizes notification of npm/PyPI availability.
