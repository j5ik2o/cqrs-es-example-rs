#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
DIST_DIR="$ROOT_DIR/dist/lambda/read-model-updater"
ARTIFACT_PATH="$DIST_DIR/bootstrap"
ZIP_PATH="$DIST_DIR/bootstrap.zip"

mkdir -p "$DIST_DIR"

# Linuxコンテナ上でビルドし、Lambda互換のx86_64バイナリを生成する。
docker run --rm \
  --platform linux/amd64 \
  --env SQLX_OFFLINE=${SQLX_OFFLINE:-1} \
  -v "$ROOT_DIR":/workspace \
  -w /workspace \
  rust:1.80 \
  bash -lc '
set -euo pipefail
# ensure cargo is on PATH
source /usr/local/cargo/env
cargo build --package read-model-updater --release
strip target/release/read-model-updater || true
'

cp "$ROOT_DIR/target/release/read-model-updater" "$ARTIFACT_PATH"

python3 - "$ARTIFACT_PATH" "$ZIP_PATH" <<'PY'
import pathlib
import sys
import zipfile

artifact = pathlib.Path(sys.argv[1])
zip_path = pathlib.Path(sys.argv[2])
with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
    zf.write(artifact, "bootstrap")
PY

cat <<EOM
Lambda用バイナリを生成しました: $ARTIFACT_PATH
デプロイ用ZIPを生成しました: $ZIP_PATH
EOM
