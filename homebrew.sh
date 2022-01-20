name=grant
version=0.0.1-beta.4

cargo build --release
cd target/release
tar -czf $name-$version-x86_64-apple-darwin.tar.gz $name
shasum -a 256 $name-$version-x86_64-apple-darwin.tar.gz
