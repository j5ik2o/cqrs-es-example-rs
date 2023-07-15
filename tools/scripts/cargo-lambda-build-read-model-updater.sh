#!/bin/sh
set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../

rm -fr target/lambda/release/cqrs-es-example-read-model-updater
cargo lambda build -p cqrs-es-example-read-model-updater --release --output-format zip
MD5=$(md5 -q target/lambda/cqrs-es-example-read-model-updater/bootstrap.zip)
cp target/lambda/cqrs-es-example-read-model-updater/bootstrap.zip target/lambda/cqrs-es-example-read-model-updater/bootstrap-${MD5}.zip
echo "target/lambda/cqrs-es-example-read-model-updater/bootstrap-${MD5}.zip"
echo "MD5=${MD5}"

popd