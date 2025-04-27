## suexrs - Command Execution with Different Privileges

A lightweight privilege switching tool for executing commands with different user and group permissions. Think of it as a streamlined alternative to `sudo` and `su`. 

`suexrs` allows you to run commands with different user and group privileges. Unlike traditional tools like `su` or `sudo`, `suexrs` executes programs directly rather than as child processes, which provides better handling of TTY and signals.

#### Key Features

- Direct program execution (not spawning child processes)
- Dual privilege management model:
  - For root users: Ability to step down to lower privileges (similar to `su`)
  - For non-root users in the `suexrs` group: Ability to elevate to specific privileges or switch to any other user
- Support for both username/group names and numeric uid/gid
- Group-based access control (users in the `suexrs` group can execute commands with elevated privileges)
- Simpler and more streamlined than traditional `su`/`sudo`

#### Usage

Basic syntax:

```shell
suexrs [USER[:GROUP]] COMMAND [ARGUMENTS...]
```

Where:

- `USER`: Username or numeric uid (optional for non-root users, defaults to root)
- `GROUP`: (Optional) Group name or numeric gid
- `COMMAND`: The program to execute
- `ARGUMENTS`: Any additional arguments for the command

You can also use the `@` or `+` prefix for the USER specification:

```shell
suexrs [@|+]USER[:GROUP] COMMAND [ARGUMENTS...]
```

#### Examples

For root users (stepping down in privileges):

```shell
# Run as a non-privileged user
suexrs nobody /bin/program

# Run with specific user and group
suexrs nginx:www-data /usr/sbin/nginx -c /etc/nginx/nginx.conf
```

For non-root users in the `suexrs` group (elevating or switching privileges):

```shell
# Elevate to root
suexrs root /bin/program

# Switch to a different user
suexrs webadmin /usr/bin/configure-site
```

#### Setup on host

`suexrs` requires root privileges to operate as it performs uid/gid changes. To set it up:

```shell
# Clone repository and build
git clone https://github.com/abitcipher/suexrs
cd $PWD/suexrs

# Build
make build-release
# or
cargo build --release

# Create the `suexrs` group if it doesn't exist
groupadd --system suexrs

# Set the appropriate permissions on the `suexrs` binary
chown root:suexrs $PWD/target/release/suexrs
chmod 4750 $PWD/target/release/suexrs

# Add users who should be able to use `suexrs`
usermod -a -G suexrs username

# Put `suexrs` into directory where normal user may run
cp $PWD/target/release/suexrs /usr/local/bin/
chown root:suexrs /usr/local/bin/suexrs
chmod 4750 /usr/local/bin/suexrs
```
#### Manual test
###### on host
```shell
## download
git clone https://github.com/abitcipher/suexrs
cd $PWD/suexrs

## build docker image 
make docker-build
make docker-run
```
###### inside docker
```shell
test@9aba56cd8adb:/app$ suexrs root passwd root
New password: 
Retype new password: 
passwd: password updated successfully

test@9aba56cd8adb:/app$ su root
Password: 
root@9aba56cd8adb:/app#
```
---
#### Attribution

`suexrs` is a reimplementation of [`su-exec`](https://github.com/ncopa/su-exec), enhanced for improved usability and maintainability.
