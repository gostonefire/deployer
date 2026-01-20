# Configure for common.sh

`common.sh` is used by all deployment scripts, including `master_deploy.sh`. It contains the run_cmd() function.
For all other scripts to work the `common.sh` file must be in a path that all users naturally has the path to, i.e. `/usr/local/bin`.

* Copy `common.sh` to `/usr/local/bin` with sudo.
* Modify permissions with `sudo chmod 755 /usr/local/bin/common.sh`
