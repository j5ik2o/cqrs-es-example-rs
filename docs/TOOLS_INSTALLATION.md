# Tools Installation

## Rust

以下のコマンドを実行し、すべてdefaultの選択でインストールする。

```shell
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

https://www.rust-lang.org/ja/tools/install

## asdf

```sh
$ brew install asdf
$ echo -e "\n. $(brew --prefix asdf)/libexec/asdf.sh" >> ${ZDOTDIR:-~}/.zshrc
```

https://asdf-vm.com/

For Ubuntu, install according to the following.

https://asdf-vm.com/guide/getting-started.html

## jq

```shell
$ asdf plugin-add jq https://github.com/AZMCode/asdf-jq.git
$ asdf install jq 1.6
$ asdf local jq 1.6
$ jq --version
jq-1.6
```

https://github.com/ryodocx/asdf-jq

### cargo-make

```shell
$ cargo install cargo-make
```

https://github.com/sagiegurari/cargo-make

### sqlx-cli

```shell
$ cargo install sqlx-cli
```

If the above command fails, try running as `cargo install sqlx-cli --locked`.

For Ubuntu, execute the above command after executing the following.

```shell
$ sudo apt install pkg-config libssl-dev
```

https://github.com/launchbadge/sqlx/

### Docker

#### Mac

Follow the instructions below to install it.

Docker Desktop for Mac

https://docs.docker.com/desktop/install/mac-install/

#### Ubuntu

Follow the instructions below to install it.

- Docker Desktop for Ubuntu
    - https://docs.docker.com/desktop/install/debian/
- Docker Engine for Ubuntu
    - https://docs.docker.com/engine/install/ubuntu/

In addition, to be able to use it without sudo, please follow these steps.

https://qiita.com/katoyu_try1/items/1bdaaad9f64af86bbfb7
